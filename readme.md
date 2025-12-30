# Voxer

> A high-performance, high-render-distance WebGPU voxel engine

---

## Overview

**Voxer** is a WebGPU-based voxel engine with a strong focus on achieving **very large pure render distances** while maintaining **ultra-high frame rates**.

At its current stage, the engine is primarily a rendering pipeline, with temporary (yet functional) systems for networking, world generation, and related subsystems.

The engine is capable of rendering up to **32 chunks per axis from the camera center**  
(chunk size: `16³` voxels), **without LOD**, while sustaining frame rates suitable for **240–480 Hz** displays.

This is achieved through a combination of **CPU-side custom ultra-high-performance data structures**
(specifically designed for single-threaded, O(1), near-zero-overhead operations to avoid CPU saturation)
and a **heavily GPU-driven architecture**, minimizing per-frame CPU involvement (<5% CPU overhead per frame in worst-case scenarios),
therefore leaving the CPU almost entirely available for large-radius world generation and other non-rendering tasks.

**Pure render distance** refers to rendering distance of the full-resolution voxel world, no LOD

---

## Features

- **GPU-driven rendering pipeline**  

- **Near-fully GPU-side chunk meshing**  
  (culled, non-greedy)

- **Near-fully GPU-side culling**
    - Camera-relative back-face culling
    - View frustum culling
    - Occlusion culling

- **Fast, deterministic, chunk-based world generation**  
  (`16³` voxels per chunk, with the FastNoise2 crate)

- **Minimal external dependencies with custom high-performance data structures**  
  (memory allocators, maps, unique-vectors, etc.)

---
