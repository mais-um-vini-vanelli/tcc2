@tool
class_name MaterialSection extends Section

func bind(binder: Binder, mat: SceneMaterial) -> void:
	_channel(
		binder,
		mat.albedo,
		mat.albedo.rgb,
		%albedo_kind,
		%albedo_constant,
		%albedo_factor,
		%albedo_source,
		%albedo_image)

	_channel(
		binder,
		mat.metallic,
		mat.metallic.scalar,
		%metallic_kind,
		%metallic_constant,
		%metallic_factor,
		%metallic_source,
		%metallic_image,
		%metallic_channel)

	_channel(
		binder,
		mat.roughness,
		mat.roughness.scalar,
		%roughness_kind,
		%roughness_constant,
		%roughness_factor,
		%roughness_source,
		%roughness_image,
		%roughness_channel)

	_channel(
		binder,
		mat.emissive,
		mat.emissive.rgb,
		%emissive_kind,
		%emissive_constant,
		%emissive_factor,
		%emissive_source,
		%emissive_image)

	binder.bind(%normal_source, mat.normal, &"source")
	binder.bind(%normal_image, mat.normal, &"image")
	binder.bind(%normal_scale, mat, &"normal_scale")

	binder.follow(mat.normal.changed, func():
		var mapped := not mat.normal.source.is_empty()

		show_row(%normal_image, mapped
			and mat.normal.source.get_extension().to_lower() == "glb")
		show_row(%normal_scale, mapped))

	binder.bind(%transparency, mat, &"transparency")
	binder.bind(%refractive_index, mat, &"refractive_index")

func _channel(
	binder: Binder,
	value: SceneChannel,
	holder: Object,
	kind: Control,
	constant: Control,
	factor: Control,
	source: Control,
	image: Control,
	channel: Control = null) -> void:
	binder.bind(kind, value, &"kind")
	binder.bind(constant, holder, &"constant")
	binder.bind(factor, holder, &"factor")
	binder.bind(source, value.texture, &"source")
	binder.bind(image, value.texture, &"image")

	if channel != null:
		binder.bind(channel, holder, &"channel")

	binder.follow(value.changed, func():
		var textured := value.kind == SceneChannel.Kind.TEXTURE

		show_row(constant, not textured)
		show_row(factor, textured)
		show_row(source, textured)
		show_row(image, textured and value.texture.source.get_extension().to_lower() == "glb")

		if channel != null:
			show_row(channel, textured))
