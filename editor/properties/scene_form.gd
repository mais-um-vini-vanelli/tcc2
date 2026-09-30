class_name Tcc2SceneForm extends Tcc2Form

func _build() -> void:
	bind(%transform_method, node, &"transform_method")

	bind(%backface_cull, node, &"backface_cull")
	bind(%wireframe, node, &"wireframe")
	bind(%perspective_correct, node, &"perspective_correct")
	bind(%depth_test, node, &"depth_test")

	bind(%clipping_enabled, node, &"clipping_enabled")

	bind(%mipmaps, node, &"mipmaps")
	bind(%filtering, node, &"filtering")

	bind(%shading_frequency, node, &"shading_frequency")
	bind(%reflectance_model, node, &"reflectance_model")

	bind(%max_depth, node, &"max_depth")
	bind(%shadows, node, &"shadows")
	bind(%reflections, node, &"reflections")
	bind(%refractions, node, &"refractions")

	bind(%bvh, node, &"bvh")
	bind(%bvh_split, node, &"bvh_split")

	bind(%width, node, &"width")
	bind(%height, node, &"height")
	bind(%background_enabled, node, &"background_enabled")
	bind(%background, node, &"background")
	bind(%exposure, node, &"exposure")
	bind(%tonemap, node, &"tonemap")
