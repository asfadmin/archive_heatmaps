extern crate earcutr;
use std::collections::VecDeque;

use geo::geometry::{Coord, LineString, Polygon};
use geo::{Simplify, TriangulateEarcut, coord};

use crate::canvas::geometry::BlendVertex;
use crate::ingest::load::BufferStorage;
use crate::ingest::load::Data;

/// Converts the passed data into a triangular mesh using the earcutting algorithm,
///     this is done for a varying level of detail to allow for LODs, polygon simplification
///     is done using the Ramer-Douglas-Peucker algorithm
pub fn mesh_data(data_exterior: Data) -> Vec<BufferStorage> {
    let mut positions: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut weights: Vec<u64> = Vec::new();

    match data_exterior {
        Data::Outline(outline_data) => {
            for poly in outline_data {
                positions.push(poly.exterior().points().map(|x| (x.x(), x.y())).collect());
                weights.push(0);
            }
        }

        Data::Heatmap(heatmap_data) => {
            for gran in heatmap_data {
                positions.push(
                    gran.geometry
                        .exterior()
                        .points()
                        .map(|x| (x.x(), x.y()))
                        .collect(),
                );
                weights.push(gran.weight);
            }
        }
    }

    let mut lods: Vec<BufferStorage> = Vec::new();

    let mut polygons: Vec<Polygon> = positions
        .iter()
        .map(|poly| {
            poly.iter()
                .map(|(x, y)| {
                    coord! {x: *x, y: *y}
                })
                .collect()
        })
        .map(|mut exterior: Vec<Coord>| {
            // Last entry is a duplicate of the first
            let _ = exterior.pop();
            Polygon::new(LineString(exterior.clone()), Vec::new())
        })
        .collect();

    let mut level: usize = 0;
    while level <= 2 {
        let mut weights = VecDeque::from(weights.clone());
        let mut total_vertices: Vec<BlendVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        for poly in &mut polygons {
            let simplified = poly.simplify(&(0.2 * level as f64));
            // Run the ear cutting algorithm, triangles contains a list of indices after
            let triangles_raw = simplified.earcut_triangles_raw();

            // Append current indices to the end of prior indices with offset
            let offset = total_vertices.len();
            for indice in &triangles_raw.triangle_indices {
                indices.push(
                    (indice + offset)
                        .try_into()
                        .expect("ERROR: Failed to convert usize to u32"),
                );
            }

            // Place data for each vertex into a vertex struct
            let weight = weights
                .pop_front()
                .expect("Weights was not equal to the number of polygons");
            let mut i = 0;
            while i < triangles_raw.vertices.len() {
                total_vertices.push(BlendVertex {
                    position: [
                        triangles_raw.vertices[i] as f32,
                        triangles_raw.vertices[i + 1] as f32,
                        0.0,
                    ],
                    weight: u32::try_from(weight).expect("Failed to convert weight to u32"),
                });

                i += 2;
            }
        }

        let num_indices = indices
            .len()
            .try_into()
            .expect("ERROR: Failed to convert usize into u32");

        lods.push(BufferStorage {
            vertices: total_vertices,
            indices,
            num_indices,
        });

        level += 1;
    }
    lods
}
