class_name Read

var _path: String
var _json := {}
var _bin := PackedByteArray()
var _scene: SceneRoot
var _image_base := 0
var _mesh_base := 0
var _count := 0
var _warnings: Array[String] = []

static func from_tcc2(path: String, scene: SceneRoot,
		warnings: Array[String] = []) -> bool:
	var file := FileAccess.open(path, FileAccess.READ)

	if file == null:
		push_error("could not read %s: %s" % [path,
			error_string(FileAccess.get_open_error())])
		return false

	var bytes := file.get_buffer(file.get_length())
	file.close()

	if bytes.size() < 12 or bytes.decode_u32(0) != Write.MAGIC:
		push_error("%s is not a tcc2 scene" % path)
		return false

	var version := bytes.decode_u32(4)
	var length := bytes.decode_u32(8)

	if version != Write.VERSION:
		push_error("%s is version %d, not %d" % [path, version, Write.VERSION])
		return false
	elif 12 + length > bytes.size():
		push_error("%s has a truncated header" % path)
		return false

	var parsed = JSON.parse_string(
		bytes.slice(12, 12 + length).get_string_from_utf8())

	if not (parsed is Dictionary):
		push_error("%s has no readable header" % path)
		return false

	var reader := Read.new()
	reader._path = path
	reader._json = parsed
	reader._bin = bytes.slice(12 + length)
	reader._scene = scene
	reader._image_base = scene.assets.image_count()
	reader._mesh_base = scene.assets.mesh_count()

	reader._read_images()
	reader._read_meshes()

	for entry in reader._json.get("roots", []):
		if entry is Dictionary:
			scene.add_child(reader._node(entry))

	reader._read_parameters()
	warnings.append_array(reader._warnings)

	return true

func _read_images() -> void:
	var list: Array = _json.get("images", [])

	for i in list.size():
		var image := Image.new()

		if image.load_png_from_buffer(_slice(list[i])) != OK:
			_warn("image %d could not be read" % i)
			image = Image.create_empty(1, 1, false, Image.FORMAT_RGBA8)

		_scene.assets.add_image(image, SceneAssets.label(_path, str(i)))

func _read_meshes() -> void:
	var list: Array = _json.get("meshes", [])

	for i in list.size():
		_scene.assets.add_mesh(_mesh(list[i]), SceneAssets.label(_path, str(i)))

func _mesh(entry: Dictionary) -> Mesh:
	var arrays := []
	arrays.resize(Mesh.ARRAY_MAX)

	arrays[Mesh.ARRAY_VERTEX] = _vec3(entry.get("vertices", {}))
	arrays[Mesh.ARRAY_INDEX] = _indices(entry.get("indices", {}))

	if entry.has("normals"):
		arrays[Mesh.ARRAY_NORMAL] = _vec3(entry["normals"])

	if entry.has("tangents"):
		arrays[Mesh.ARRAY_TANGENT] = _slice(entry["tangents"]).to_float32_array()

	if entry.has("uvs"):
		arrays[Mesh.ARRAY_TEX_UV] = _vec2(entry["uvs"])

	var built := ArrayMesh.new()
	built.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arrays)

	return built

func _indices(slice: Dictionary) -> PackedInt32Array:
	var indices := _slice(slice).to_int32_array()

	var i := 0

	while i + 2 < indices.size():
		var keep := indices[i + 1]
		indices[i + 1] = indices[i + 2]
		indices[i + 2] = keep
		i += 3

	return indices

func _slice(slice: Dictionary) -> PackedByteArray:
	var at := int(slice.get("offset", 0))
	var length := int(slice.get("length", 0))

	if at < 0 or length < 0 or at + length > _bin.size():
		_warn("a slice falls outside the blob")
		return PackedByteArray()

	return _bin.slice(at, at + length)

func _vec3(slice: Dictionary) -> PackedVector3Array:
	var values := _slice(slice).to_float32_array()
	var out := PackedVector3Array()
	out.resize(values.size() / 3)

	for i in out.size():
		out[i] = Vector3(values[i * 3], values[i * 3 + 1], values[i * 3 + 2])

	return out

func _vec2(slice: Dictionary) -> PackedVector2Array:
	var values := _slice(slice).to_float32_array()
	var out := PackedVector2Array()
	out.resize(values.size() / 2)

	for i in out.size():
		out[i] = Vector2(values[i * 2], values[i * 2 + 1])

	return out

func _node(entry: Dictionary) -> SceneObject:
	var node := _object(entry)
	var label = entry.get("name")

	_count += 1
	node.name = String(label) if label is String and not label.is_empty() else "Node%d" % _count

	node.rotation_order = EULER_ORDER_ZYX

	var transform: Dictionary = entry.get("transform", {})
	node.position = _vec(transform.get("t", []), Vector3.ZERO)
	node.rotation_degrees = _vec(transform.get("r", []), Vector3.ZERO)
	node.scale = _vec(transform.get("s", []), Vector3.ONE)

	for child in entry.get("children", []):
		if child is Dictionary:
			node.add_child(_node(child))

	return node

func _object(entry: Dictionary) -> SceneObject:
	if entry.has("camera"):
		return _camera(entry["camera"])

	if entry.has("light"):
		return _light(entry["light"])

	if entry.has("geometry"):
		return _geometry(entry["geometry"], entry.get("material", {}))

	return SceneObject.new()

func _camera(entry: Dictionary) -> SceneCamera:
	var node := SceneCamera.new()

	if entry.has("orthographic"):
		node.kind = SceneCamera.Kind.ORTHOGRAPHIC

	var body: Dictionary = entry.get("orthographic", entry.get("perspective", {}))
	var model = node.model()

	if node.kind == SceneCamera.Kind.ORTHOGRAPHIC:
		model.height = float(body.get("height", 1.0))
	else:
		model.fov = float(body.get("fov", 60.0))

	model.near = float(body.get("near", 0.1))
	model.far = float(body.get("far", 100.0))

	return node

func _light(entry: Dictionary) -> SceneLight:
	const KINDS := {
		"point": SceneLight.Kind.POINT,
		"directional": SceneLight.Kind.DIRECTIONAL,
		"spot": SceneLight.Kind.SPOT,
		"environment": SceneLight.Kind.ENVIRONMENT,
	}

	var node := SceneLight.new()

	for key in KINDS:
		if not entry.has(key):
			continue

		var body: Dictionary = entry[key]
		node.kind = KINDS[key]

		var model = node.model()
		model.color = _color(body.get("color", []))
		model.intensity = float(body.get("intensity", 1.0))

		if key == "spot":
			model.inner_angle = float(body.get("inner_angle", 20.0))
			model.outer_angle = float(body.get("outer_angle", 30.0))

		break

	return node

func _geometry(entry: Dictionary, material: Dictionary) -> SceneGeometry:
	var node := SceneGeometry.new()

	if entry.has("cube"):
		node.kind = SceneGeometry.Kind.CUBE
		node.cube.half_edge = float(entry["cube"].get("half_edge", 0.5))
	elif entry.has("triangle"):
		node.kind = SceneGeometry.Kind.TRIANGLE
		var points: Array = entry["triangle"].get("points", [])

		if points.size() == 3:
			node.triangle.a = _vec(points[0], Vector3.ZERO)
			node.triangle.b = _vec(points[1], Vector3.ZERO)
			node.triangle.c = _vec(points[2], Vector3.ZERO)
	elif entry.has("mesh"):
		node.kind = SceneGeometry.Kind.MESH
		node.mesh.index = _mesh_base + int(entry["mesh"])
	else:
		node.kind = SceneGeometry.Kind.SPHERE
		node.sphere.radius = float(entry.get("sphere", {}).get("radius", 0.5))

	node.material = _material(material)

	return node

func _material(entry: Dictionary) -> SceneMaterial:
	var material := SceneMaterial.new()

	_channel(material.albedo, entry.get("albedo", {}), true)
	_channel(material.metallic, entry.get("metallic", {}), false)
	_channel(material.roughness, entry.get("roughness", {}), false)
	_channel(material.emissive, entry.get("emissive", {}), true)

	material.emissive_strength = float(entry.get("emissive_strength", 1.0))
	material.transparency = float(entry.get("transparency", 0.0))
	material.refractive_index = float(entry.get("refractive_index", 1.5))

	var normal: Dictionary = entry.get("normal", {})

	if normal.has("map"):
		material.normal = _image_base + int(normal["map"])
		material.normal_scale = float(normal.get("scale", 1.0))

	return material

func _channel(channel: SceneChannel, entry: Dictionary, rgb: bool) -> void:
	if entry.has("map"):
		channel.kind = SceneChannel.Kind.MAP
		channel.map = _image_base + int(entry["map"])
		return

	if rgb:
		channel.rgb.constant = _color(entry.get("constant", []))
	else:
		channel.scalar.constant = float(entry.get("constant", 0.0))

func _read_parameters() -> void:
	var preset: Dictionary = _json.get("parameters", {})

	if preset.is_empty():
		return

	var pick := func(names: Array, value, fallback: int) -> int:
		var index: int = names.find(value)
		return index if index >= 0 else fallback

	var raster: Dictionary = preset.get("rasterization", {})
	_scene.backface_cull = raster.get("backface_cull", _scene.backface_cull)
	_scene.perspective_correct = raster.get("perspective_correct",
		_scene.perspective_correct)
	_scene.depth_test = raster.get("depth_test", _scene.depth_test)

	var shading: Dictionary = preset.get("shading", {})
	_scene.reflectance_model = pick.call(["blinn_phong"],
		shading.get("reflectance_model"), _scene.reflectance_model)
	_scene.texture_filter = pick.call(
		["nearest", "bilinear", "mipmap_bilinear", "trilinear"],
		shading.get("texture_filter"), _scene.texture_filter)

	var tracing: Dictionary = preset.get("ray_tracing", {})
	_scene.max_depth = int(tracing.get("max_depth", _scene.max_depth))
	_scene.shadows = tracing.get("shadows", _scene.shadows)
	_scene.reflections = tracing.get("reflections", _scene.reflections)
	_scene.refractions = tracing.get("refractions", _scene.refractions)
	_scene.acceleration = pick.call(["brute_force", "bvh_median", "bvh_sah"],
		tracing.get("acceleration"), _scene.acceleration)

	var output: Dictionary = preset.get("output", {})
	_scene.width = int(output.get("width", _scene.width))
	_scene.height = int(output.get("height", _scene.height))
	_scene.exposure = float(output.get("exposure", _scene.exposure))
	_scene.tonemap = pick.call(["none", "reinhard"], output.get("tonemap"),
		_scene.tonemap)

func _vec(values, fallback: Vector3) -> Vector3:
	if not (values is Array) or values.size() < 3:
		return fallback

	return Vector3(float(values[0]), float(values[1]), float(values[2]))

func _color(values) -> Color:
	if not (values is Array) or values.size() < 3:
		return Color.WHITE

	return Color(float(values[0]), float(values[1]),
		float(values[2])).linear_to_srgb()

func _warn(message: String) -> void:
	_warnings.append(message)
	push_warning(message)
