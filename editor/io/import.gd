class_name Import

const LUMENS_PER_WATT := 683.0

static var _documents := {}

var _path: String
var _state: GLTFState
var _assets: SceneAssets
var _cameras: bool
var _json := {}
var _warnings: Array[String] = []

static func glb(path: String, cameras: bool, assets: SceneAssets,
		warnings: Array[String] = []) -> SceneObject:
	var importer := _reader(path, assets)

	if importer == null:
		return null

	importer._cameras = cameras

	var group := SceneObject.new()
	group.name = path.get_file().get_basename().trim_suffix(".tcc2")

	for index in importer._state.root_nodes:
		importer._build(index, group)

	warnings.append_array(importer._warnings)

	return group

static func images(path: String, assets: SceneAssets) -> int:
	var first := assets.find_image(SceneAssets.label(path, "0"))

	if first >= 0:
		return first

	var pictures := _pictures(path)

	for i in pictures.size():
		var picture: Texture2D = pictures[i]
		var image: Image = picture.get_image() if picture != null else null

		if image == null:
			continue

		var index := assets.add_image(_rgba(image), SceneAssets.label(path, str(i)))

		if first < 0:
			first = index

	return first

static func meshes(path: String, assets: SceneAssets) -> int:
	var importer := _reader(path, assets)

	if importer == null:
		return -1

	var first := -1

	for mesh in importer._mesh_count():
		for surface in importer._surfaces(mesh).get_surface_count():
			if importer._arrays(mesh, surface).is_empty():
				continue

			var index := importer._mesh_index(mesh, surface)

			if first < 0:
				first = index

	return first

static func _reader(path: String, assets: SceneAssets) -> Import:
	var parsed := _parsed(path)

	if parsed == null:
		return null

	var importer := Import.new()
	importer._path = path
	importer._state = parsed
	importer._assets = assets
	importer._json = parsed.json

	return importer

static func _parsed(path: String) -> GLTFState:
	if not _documents.has(path):
		var doc := GLTFDocument.new()
		var parsed := GLTFState.new()

		if doc.append_from_file(path, parsed) != OK:
			push_error("could not read %s" % path)
			parsed = null

		_documents[path] = parsed

	return _documents[path]

static func _pictures(path: String) -> Array:
	if path.get_extension().to_lower() == "glb":
		var parsed := _parsed(path)
		return parsed.get_images() if parsed != null else []

	var picture := Image.load_from_file(path)

	if picture == null:
		push_error("could not read %s" % path)
		return []

	return [ImageTexture.create_from_image(picture)]

func _build(node_index: int, parent: SceneObject) -> void:
	var gltf: GLTFNode = _state.get_nodes()[node_index]
	var node_name: String = gltf.resource_name

	if node_name.is_empty():
		node_name = "Node%d" % node_index

	var node := _node(gltf, node_name)
	node.name = node_name
	node.transform = gltf.xform
	parent.add_child(node)

	for child in gltf.children:
		_build(child, node)

func _node(gltf: GLTFNode, node_name: String) -> SceneObject:
	if gltf.mesh >= 0:
		return _mesh(gltf.mesh, node_name)

	if gltf.camera >= 0 and _cameras:
		_cameras = false
		return _camera(_state.get_cameras()[gltf.camera])

	if gltf.light >= 0:
		return _light(_state.get_lights()[gltf.light])

	return SceneObject.new()

func _mesh(mesh: int, node_name: String) -> SceneObject:
	var count := _surfaces(mesh).get_surface_count()

	if count == 1:
		if not _usable(mesh, 0, node_name):
			return SceneObject.new()

		return _primitive(mesh, 0, node_name)

	var group := SceneObject.new()

	for surface in count:
		if not _usable(mesh, surface, node_name):
			continue

		var primitive := _primitive(mesh, surface, node_name)
		primitive.name = "%s_%d" % [node_name, surface]
		group.add_child(primitive)

	return group

func _mesh_count() -> int:
	return _state.get_meshes().size()

func _surfaces(mesh: int) -> ImporterMesh:
	return _state.get_meshes()[mesh].mesh

func _arrays(mesh: int, index: int) -> Array:
	var surfaces := _surfaces(mesh)

	if index < 0 or index >= surfaces.get_surface_count():
		return []

	if surfaces.get_surface_primitive_type(index) != Mesh.PRIMITIVE_TRIANGLES:
		return []

	var arrays := surfaces.get_surface_arrays(index)
	var vertices: PackedVector3Array = arrays[Mesh.ARRAY_VERTEX]

	return [] if vertices.is_empty() else arrays

func _mesh_index(mesh: int, index: int) -> int:
	var label := SceneAssets.label(_path, "%d.%d" % [mesh, index])
	var found := _assets.find_mesh(label)

	if found >= 0:
		return found

	var built := ArrayMesh.new()
	built.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, _arrays(mesh, index))

	return _assets.add_mesh(built, label)

func _usable(mesh: int, index: int, node_name: String) -> bool:
	if not _arrays(mesh, index).is_empty():
		return true

	_warn("%s: mesh %d surface %d has no triangles, dropped"
		% [node_name, mesh, index])

	return false

func _warn_ignored(mesh: int, index: int, node_name: String) -> void:
	var arrays := _arrays(mesh, index)

	const IGNORED_ARRAYS := {
		Mesh.ARRAY_TEX_UV2: "a second uv set is ignored",
		Mesh.ARRAY_COLOR: "vertex colors are ignored",
		Mesh.ARRAY_BONES: "a skin is ignored",
	}

	for array in IGNORED_ARRAYS:
		if arrays[array] != null:
			_warn("%s: %s" % [node_name, IGNORED_ARRAYS[array]])

func _primitive(mesh: int, index: int, node_name: String) -> SceneGeometry:
	_warn_ignored(mesh, index, node_name)

	var primitive := SceneGeometry.new()
	primitive.kind = SceneGeometry.Kind.MESH
	primitive.mesh.index = _mesh_index(mesh, index)
	primitive.material = _material(mesh, index)

	return primitive

func _camera(gltf: GLTFCamera) -> SceneCamera:
	var node := SceneCamera.new()

	if gltf.perspective:
		node.kind = SceneCamera.Kind.PERSPECTIVE
		node.perspective.fov = rad_to_deg(gltf.fov)
	else:
		node.kind = SceneCamera.Kind.ORTHOGRAPHIC
		node.orthographic.height = gltf.size_mag * 2.0
	
	var mod = node.model()
	mod.near = gltf.depth_near
	mod.far = gltf.depth_far

	return node

func _light(gltf: GLTFLight) -> SceneLight:
	var node := SceneLight.new()

	match gltf.light_type:
		"directional":
			node.kind = SceneLight.Kind.DIRECTIONAL

		"spot":
			node.kind = SceneLight.Kind.SPOT
			node.spot.inner_angle = rad_to_deg(gltf.inner_cone_angle)
			node.spot.outer_angle = rad_to_deg(gltf.outer_cone_angle)

		_:
			node.kind = SceneLight.Kind.POINT

	var model = node.model()
	model.color = gltf.color.linear_to_srgb()
	model.intensity = gltf.intensity / LUMENS_PER_WATT

	return node

func _material(mesh: int, index: int) -> SceneMaterial:
	var mat := _surfaces(mesh).get_surface_material(index)
	var id: int = _state.get_materials().find(mat)

	if id < 0 or not (mat is BaseMaterial3D):
		return SceneMaterial.new()

	var _mat := mat as BaseMaterial3D
	var material := SceneMaterial.new()

	var tint := Color(_mat.albedo_color, 1.0)

	if _mat.albedo_texture == null:
		material.albedo.rgb.constant = tint
	else:
		_set_map(material.albedo, _mat.albedo_texture, "albedo",
			func(image: Image) -> Image: return _tinted(image, tint))

	_set_scalar(material.metallic, _mat.metallic_texture, _mat.metallic,
		_mat.metallic_texture_channel, "metallic")

	_set_scalar(material.roughness, _mat.roughness_texture, _mat.roughness,
		_mat.roughness_texture_channel, "roughness")

	if _mat.emission_enabled:
		var emission := _mat.emission
		material.emissive_strength = _mat.emission_energy_multiplier

		if _mat.emission_texture == null:
			material.emissive.rgb.constant = emission
		else:
			_set_map(material.emissive, _mat.emission_texture, "emissive",
				func(image: Image) -> Image: return _tinted(image, emission))

	if _mat.normal_enabled:
		material.normal = _index_of(_mat.normal_texture, "normal",
			func(image: Image) -> Image: return image)
		material.normal_scale = _mat.normal_scale

	if _mat.uv1_offset != Vector3.ZERO or _mat.uv1_scale != Vector3.ONE:
		_warn("%s: a uv offset or scale was dropped" % _path)

	var extensions := _extensions("materials", id)

	material.transparency = float(extensions
		.get("KHR_materials_transmission", {}).get("transmissionFactor", 0.0))

	material.refractive_index = float(extensions
		.get("KHR_materials_ior", {}).get("ior", 1.5))

	return material

func _set_map(channel: SceneChannel, picture: Texture2D, label: String,
		bake: Callable) -> void:
	var index := _index_of(picture, label, bake)

	if index < 0:
		return

	channel.kind = SceneChannel.Kind.MAP
	channel.map = index

func _set_scalar(channel: SceneChannel, picture: Texture2D, factor: float,
		from_channel: int, label: String) -> void:
	var index := -1

	if picture != null:
		index = _index_of(picture, label,
			func(image: Image) -> Image: return _extract(image, from_channel, factor))

	if index < 0:
		channel.scalar.constant = factor
		return

	channel.kind = SceneChannel.Kind.MAP
	channel.map = index

func _index_of(picture: Texture2D, label: String, bake: Callable) -> int:
	if picture == null:
		return -1

	var found: int = _pictures(_path).find(picture)

	if found < 0:
		_warn("%s: a texture is not one of the file images" % _path)
		return -1

	var image := picture.get_image()

	if image == null:
		return -1

	return _assets.add_image(bake.call(_rgba(image)),
		SceneAssets.label(_path, "%d %s" % [found, label]))

static func _rgba(image: Image) -> Image:
	var copy := Image.new()
	copy.copy_from(image)

	if copy.is_compressed():
		copy.decompress()

	copy.convert(Image.FORMAT_RGBA8)

	return copy

func _tinted(image: Image, tint: Color) -> Image:
	if tint.r == 1.0 and tint.g == 1.0 and tint.b == 1.0:
		return image

	return Image.create_from_data(image.get_width(), image.get_height(), false,
		Image.FORMAT_RGBA8,
		Tcc2Bake.apply_rgb_factor(image.get_data(), tint.srgb_to_linear()))

func _extract(image: Image, from_channel: int, factor: float) -> Image:
	var channel := 0

	match from_channel:
		BaseMaterial3D.TEXTURE_CHANNEL_GREEN:
			channel = 1

		BaseMaterial3D.TEXTURE_CHANNEL_BLUE:
			channel = 2

		BaseMaterial3D.TEXTURE_CHANNEL_ALPHA:
			channel = 3

		BaseMaterial3D.TEXTURE_CHANNEL_GRAYSCALE:
			channel = -1

	return Image.create_from_data(image.get_width(), image.get_height(), false,
		Image.FORMAT_L8,
		Tcc2Bake.apply_scalar_factor(image.get_data(), channel, factor))

func _extensions(kind: String, index: int) -> Dictionary:
	var list: Array = _json.get(kind, [])

	if index < 0 or index >= list.size():
		return {}

	return list[index].get("extensions", {})

func _warn(message: String) -> void:
	_warnings.append(message)
	push_warning(message)
