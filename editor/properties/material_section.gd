@tool
class_name MaterialSection extends Section

func bind(form: Form, mat: SceneMaterial) -> void:
	_channel(
		form,
		mat.albedo,
		mat.albedo.rgb,
		%AlbedoKindRow,
		%AlbedoConstantRow,
		%AlbedoSourceRow,
		%AlbedoIndexRow)

	_channel(
		form,
		mat.metallic,
		mat.metallic.scalar,
		%MetallicKindRow,
		%MetallicConstantRow,
		%MetallicSourceRow,
		%MetallicIndexRow)

	_channel(
		form,
		mat.roughness,
		mat.roughness.scalar,
		%RoughnessKindRow,
		%RoughnessConstantRow,
		%RoughnessSourceRow,
		%RoughnessIndexRow)

	_channel(
		form,
		mat.emissive,
		mat.emissive.rgb,
		%EmissiveKindRow,
		%EmissiveConstantRow,
		%EmissiveSourceRow,
		%EmissiveIndexRow)

	form.bind(%EmissiveStrength, mat, ^"emissive_strength")
	form.bind(%NormalScale, mat, ^"normal_scale")

	form.asset_picker(%NormalSourceRow, %NormalIndexRow, mat, ^"normal",
		SceneAssets.Kind.IMAGE)

	var normal := func():
		if not is_inside_tree():
			return

		%NormalScaleRow.visible = mat.normal >= 0

	normal.call()
	mat.changed.connect(normal)

	form.bind(%Transparency, mat, ^"transparency")
	form.bind(%RefractiveIndex, mat, ^"refractive_index")

func _channel(
	form: Form,
	value: SceneChannel,
	holder: Object,
	kind: Row,
	constant: Row,
	source: Row,
	entry: Row) -> void:
	form.bind(kind.value(), value, ^"kind")
	form.bind(constant.value(), holder, ^"constant")

	var shown := func():
		if not is_inside_tree():
			return

		constant.visible = not value.kind == SceneChannel.Kind.MAP
		source.visible = value.kind == SceneChannel.Kind.MAP

	shown.call()
	value.changed.connect(shown)

	value.changed.connect(form.asset_picker(source, entry, value, ^"map",
		SceneAssets.Kind.IMAGE))
