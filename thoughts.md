- pipelineLayout<Vertex> should match create_mesh_pipeline::<Vertex> ?

- objasset should call graphics_pipeline::set_obj_compatible()


# todo!

1 - Optimization when:

How
- keep use of triangle_strip definition (is it correct for face in.obj ?) - Obj generation

Why
- less vertices

When
  - smoothing is enabled
  - (vp, vt, vn) are consistent across all the faces

Changes
  - vertex normal alogirthm, 

https://chatgpt.com/share/66f1ebef-23b8-8011-9049-73967bc6f9d7