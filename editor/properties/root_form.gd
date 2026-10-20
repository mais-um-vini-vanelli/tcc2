class_name RootForm extends Form

func _build() -> void:
	bind(%transform_method, node, &"transform_method")

	bind(%backface_cull, node, &"backface_cull")
	bind(%perspective_correct, node, &"perspective_correct")
	bind(%depth_test, node, &"depth_test")

	bind(%reflectance_model, node, &"reflectance_model")
	bind(%texture_filter, node, &"texture_filter")

	bind(%max_depth, node, &"max_depth")
	bind(%shadows, node, &"shadows")
	bind(%reflections, node, &"reflections")
	bind(%refractions, node, &"refractions")

	bind(%bvh_enabled, node, &"bvh_enabled")
	bind(%bvh_split, node, &"bvh_split")
	bind(%bvh_wireframe, node, &"bvh_wireframe")

	bind(%width, node, &"width")
	bind(%height, node, &"height")
	bind(%exposure, node, &"exposure")
	bind(%tonemap, node, &"tonemap")
