class_name RootForm extends Form

func _build() -> void:
	bind(%TransformMethod, node, ^"transform_method")

	bind(%BackfaceCull, node, ^"backface_cull")
	bind(%PerspectiveCorrect, node, ^"perspective_correct")
	bind(%DepthTest, node, ^"depth_test")

	bind(%ReflectanceModel, node, ^"reflectance_model")
	bind(%TextureFilter, node, ^"texture_filter")

	bind(%MaxDepth, node, ^"max_depth")
	bind(%Shadows, node, ^"shadows")
	bind(%Reflections, node, ^"reflections")
	bind(%Refractions, node, ^"refractions")

	bind(%BvhEnabled, node, ^"bvh_enabled")
	bind(%BvhSplit, node, ^"bvh_split")
	bind(%BvhWireframe, node, ^"bvh_wireframe")

	bind(%Width, node, ^"width")
	bind(%Height, node, ^"height")
	bind(%Exposure, node, ^"exposure")
	bind(%Tonemap, node, ^"tonemap")
