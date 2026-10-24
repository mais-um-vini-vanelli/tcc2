@tool
class_name MaterialSection extends Section

func bind(form: Form, mat: SceneMaterial) -> void:
	_channel(
		form,
		mat.albedo,
		mat.albedo.rgb,
		%AlbedoKindRow,
		%AlbedoConstantRow,
		%AlbedoFactorRow,
		%AlbedoSourceRow,
		%AlbedoImageRow)

	_channel(
		form,
		mat.metallic,
		mat.metallic.scalar,
		%MetallicKindRow,
		%MetallicConstantRow,
		%MetallicFactorRow,
		%MetallicSourceRow,
		%MetallicImageRow,
		%MetallicChannelRow)

	_channel(
		form,
		mat.roughness,
		mat.roughness.scalar,
		%RoughnessKindRow,
		%RoughnessConstantRow,
		%RoughnessFactorRow,
		%RoughnessSourceRow,
		%RoughnessImageRow,
		%RoughnessChannelRow)

	_channel(
		form,
		mat.emissive,
		mat.emissive.rgb,
		%EmissiveKindRow,
		%EmissiveConstantRow,
		%EmissiveFactorRow,
		%EmissiveSourceRow,
		%EmissiveImageRow)

	form.bind(%NormalSource, mat.normal, ^"source")
	form.bind(%NormalImage, mat.normal, ^"image")
	form.bind(%NormalScale, mat, ^"normal_scale")

	var normal := func():
		if not is_inside_tree():
			return

		var mapped := not mat.normal.source.is_empty()

		%NormalImageRow.visible = (mapped
			and mat.normal.source.get_extension().to_lower() == "glb")
		%NormalScaleRow.visible = mapped

	normal.call()
	mat.normal.changed.connect(normal)

	form.bind(%Transparency, mat, ^"transparency")
	form.bind(%RefractiveIndex, mat, ^"refractive_index")

func _channel(
	form: Form,
	value: SceneChannel,
	holder: Object,
	kind: Row,
	constant: Row,
	factor: Row,
	source: Row,
	image: Row,
	channel: Row = null) -> void:
	form.bind(kind.value(), value, ^"kind")
	form.bind(constant.value(), holder, ^"constant")
	form.bind(factor.value(), holder, ^"factor")
	form.bind(source.value(), value.texture, ^"source")
	form.bind(image.value(), value.texture, ^"image")

	if channel != null:
		form.bind(channel.value(), holder, ^"channel")

	var shown := func():
		if not is_inside_tree():
			return

		var textured := value.kind == SceneChannel.Kind.TEXTURE

		constant.visible = not textured
		factor.visible = textured
		source.visible = textured
		image.visible = (textured
			and value.texture.source.get_extension().to_lower() == "glb")

		if channel != null:
			channel.visible = textured

	shown.call()
	value.changed.connect(shown)
