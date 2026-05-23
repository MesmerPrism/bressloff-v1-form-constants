from __future__ import annotations

from types import SimpleNamespace

import numpy as np
import scipy.sparse
from scipy.integrate import solve_ivp


def weight_func(x, sigma1, sigma2):
    return np.exp(-(x**2) / 2 / sigma1**2) / sigma1 - np.exp(-(x**2) / 2 / sigma2**2) / sigma2


def sigmoid(x, slope=0.5):
    smaller = x < -2 / slope
    larger = x > 2 / slope
    out = np.zeros(x.shape)
    out[smaller] = 0
    out[larger] = 1
    inside = ~smaller & ~larger
    out[inside] = 1 / (1 + np.exp(-4 * slope * x[inside]))
    return out


def angle_dist(angle1, angle2):
    return np.pi / 2 - np.abs(np.pi / 2 - np.abs(angle1 - angle2) % np.pi)


def get_lateral_sigmas(r0, maxiter=100, eps=1e-7):
    sigma2 = r0
    for _ in range(maxiter):
        diff = r0 - np.sqrt((2 * sigma2 * np.log(sigma2 + 1)) / (sigma2 + 2))
        if abs(diff) < eps:
            return sigma2 / (1 + sigma2), sigma2
        sigma2 = sigma2 + diff
    raise ValueError("Reached maximum iteration count!")


def inverse_retino_cortical_map(x, y, eps=0.051, w0=0.087, alpha=3 / np.pi, beta=1.589 / 2):
    r_R = w0 / eps * np.exp(eps * x / alpha)
    theta_R = eps * y / beta
    return r_R * np.cos(theta_R), r_R * np.sin(theta_R)


class V1System:
    _base_cache = {}

    @classmethod
    def _cache_key(cls, n, m, r0):
        return (int(n), int(m), round(float(r0), 8))

    @classmethod
    def _build_base_matrices(cls, n, m, r0, lateral_cutoff=3.5):
        L = 1
        step_size = L / n
        cell_area = step_size**2
        delta_phi = np.pi / m
        sigma1, sigma2 = get_lateral_sigmas(r0)
        cutoff_radius = lateral_cutoff * sigma2
        angles = np.pi * np.arange(m) / m
        angle_weights = weight_func(
            angle_dist(0, np.subtract.outer(angles, angles)), 0.6060482974023431, 1.538382226567759
        )

        kernel_half_width = int(cutoff_radius / step_size)
        index_range = np.arange(-kernel_half_width, kernel_half_width + 1)
        x = step_size * index_range
        xx, yy = np.meshgrid(x, x)
        dist = np.sqrt(xx**2 + yy**2)
        inside = (dist <= cutoff_radius) & (dist > step_size / 2)
        kernel_index_grid = np.array([[(n, m) for m in index_range] for n in index_range], dtype=int)[inside]
        kernel_lateral_weights = weight_func(dist[inside], sigma1, sigma2) / dist[inside]
        kernel_angles = np.arctan2(yy[inside], xx[inside]) % (2 * np.pi)
        kernel_sector_indices = (((kernel_angles + delta_phi / 2) % np.pi) / delta_phi).astype(int)

        angular_rows = []
        angular_cols = []
        angular_data = []
        lateral_rows = []
        lateral_cols = []
        lateral_data = []
        to_index = lambda row, col, k: m * n * row + m * col + k

        for row in range(n):
            for col in range(n):
                for k in range(m):
                    idx = to_index(row, col, k)
                    for l in range(k + 1, m):
                        weight = angle_weights[k, l]
                        other = to_index(row, col, l)
                        angular_rows.append(idx)
                        angular_cols.append(other)
                        angular_data.append(weight / m)

                        angular_rows.append(other)
                        angular_cols.append(idx)
                        angular_data.append(weight / m)

        for row in range(n):
            for col in range(n):
                for rel_pos, weight, sector_index in zip(
                    kernel_index_grid, kernel_lateral_weights, kernel_sector_indices
                ):
                    source_row = (row + rel_pos[0]) % n
                    source_col = (col + rel_pos[1]) % n
                    out_idx = to_index(row, col, sector_index)
                    in_idx = to_index(source_row, source_col, sector_index)
                    lateral_rows.append(out_idx)
                    lateral_cols.append(in_idx)
                    lateral_data.append(weight * cell_area / delta_phi)

        total_dim = m * n**2
        angular_matrix = scipy.sparse.coo_array(
            (angular_data, (angular_rows, angular_cols)), shape=(total_dim, total_dim)
        )
        lateral_matrix = scipy.sparse.coo_array(
            (lateral_data, (lateral_rows, lateral_cols)), shape=(total_dim, total_dim)
        )
        return {
            "angular_matrix": scipy.sparse.csr_array(angular_matrix),
            "lateral_matrix": scipy.sparse.csr_array(lateral_matrix),
            "sigma1": sigma1,
            "sigma2": sigma2,
            "cutoff_radius": cutoff_radius,
            "angles": angles,
            "angle_weights": angle_weights,
        }

    def __init__(self, alpha=1, beta=0.5, mu=1, r0=None, N=50, M=4):
        self._L = 1
        if r0 is None:
            r0 = 3.2 / 50
        self._alpha = alpha
        self._beta = beta
        self._r0 = r0
        self._sigma3 = 0.6060482974023431
        self._sigma4 = 1.538382226567759
        self._mu = mu
        self._N = N
        self._M = M
        key = self._cache_key(N, M, r0)
        base = self._base_cache.get(key)
        self._cache_hit = base is not None
        if base is None:
            base = self._build_base_matrices(N, M, r0)
            self._base_cache[key] = base
        self._angular_matrix = base["angular_matrix"]
        self._lateral_matrix = base["lateral_matrix"]
        self._sigma1 = base["sigma1"]
        self._sigma2 = base["sigma2"]
        self._cutoff_radius = base["cutoff_radius"]
        self._angles = base["angles"]
        self._angle_weights = base["angle_weights"]
        self._evolution_matrix = None

    def connectivity(self, a):
        s = sigmoid(a)
        return self._mu * (self._angular_matrix @ s + self._beta * (self._lateral_matrix @ s))

    def integrate(self, *args, **kwargs):
        dadt = lambda t, a: -self._alpha * a + self.connectivity(a)
        return solve_ivp(dadt, *args, **kwargs)

    def integrate_preview(self, t_span, a0, t_eval, step=0.25):
        t0, _ = t_span
        a = np.array(a0, dtype=np.float64, copy=True)
        t_eval = np.asarray(t_eval, dtype=np.float64)
        y = np.empty((a.size, len(t_eval)), dtype=np.float32)
        current_t = float(t0)

        for frame_index, target_t in enumerate(t_eval):
            target_t = float(target_t)
            while current_t + 1e-12 < target_t:
                dt = min(step, target_t - current_t)
                # Semi-implicit decay keeps the preview stable at larger visual-exploration steps.
                a = (a + dt * self.connectivity(a)) / (1 + self._alpha * dt)
                current_t += dt
            y[:, frame_index] = a

        return SimpleNamespace(t=t_eval, y=y, success=True, message="preview")
