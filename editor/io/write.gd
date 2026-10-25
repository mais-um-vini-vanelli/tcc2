class_name Write

const MAGIC := 0x32434354
const VERSION := 1

var _json := {}
var _bin := PackedByteArray()
var _meshes := {}
var _images := {}
var _scene: SceneRoot
var _warnings: Array[String] = []

static func to_tcc2(scene: SceneRoot, warnings: Array[String] = []) -> PackedByteArray:
	var writer := Write.new()
	var bytes := writer._build(scene)
	warnings.assign(writer._warnings)
	return bytes

func _build(scene: SceneRoot) -> PackedByteArray:
	_scene = scene

	_json = {
		"roots": [],
		"meshes": [],
		"images": [],
		"parameters": _parameters(scene),
	}

	for child in scene.get_children():
		if child is SceneObject:
			_json["roots"].append(_node(child))

	return _encode()

func _encode() -> PackedByteArray:
	var chunk := JSON.stringify(_json, "", false, true).to_utf8_buffer()

	while chunk.size() % 4 != 0:
		chunk.append(0x20)

	var out := StreamPeerBuffer.new()
	out.big_endian = false

	out.put_u32(MAGIC)
	out.put_u32(VERSION)
	out.put_u32(chunk.size())
	out.put_data(chunk)
	out.put_data(_bin)

	return out.data_array

func _node(node: SceneObject) -> Dictionary:
	var entry := {
		"transform": {
			"t": _vec(node.position),
			"r": _vec(node.rotation_degrees),
			"s": _vec(node.scale),
		},
	}

	if not node.name.is_empty():
		entry["name"] = String(node.name)

	if node is SceneCamera:
		entry["camera"] = _camera(node)
	elif node is SceneLight:
		entry["light"] = _light(node)
	elif node is SceneGeometry:
		var geometry = _geometry(node)

		if geometry != null:
			entry["geometry"] = geometry
			entry["material"] = _material(node.material)

	var children: Array[Dictionary] = []

	for child in node.get_children():
		if child is SceneObject:
			children.append(_node(child))

	if not children.is_empty():
		entry["children"] = children

	return entry

func _camera(node: SceneCamera) -> Dictionary:
	var model = node.model()

	if node.kind == SceneCamera.Kind.PERSPECTIVE:
		return {"perspective": {"fov": model.fov, "near": model.near,
			"far": model.far}}

	return {"orthographic": {"height": model.height, "near": model.near,
		"far": model.far}}

func _light(node: SceneLight) -> Dictionary:
	var model = node.model()

	var body := {"color": _linear(model.color), "intensity": model.intensity}

	match node.kind:
		SceneLight.Kind.DIRECTIONAL:
			return {"directional": body}

		SceneLight.Kind.ENVIRONMENT:
			return {"environment": body}

		SceneLight.Kind.SPOT:
			body["inner_angle"] = model.inner_angle
			body["outer_angle"] = model.outer_angle
			return {"spot": body}

	return {"point": body}

func _geometry(node: SceneGeometry):
	match node.kind:
		SceneGeometry.Kind.SPHERE:
			return {"sphere": {"radius": node.sphere.radius}}

		SceneGeometry.Kind.CUBE:
			return {"cube": {"half_edge": node.cube.half_edge}}

		SceneGeometry.Kind.TRIANGLE:
			var a := node.triangle.a
			var b := node.triangle.b
			var c := node.triangle.c

			if (b - a).cross(c - a).length_squared() == 0.0:
				_warn("%s: the three points fall in a line, dropped" % node.name)
				return null

			return {"triangle": {"points": [_vec(a), _vec(b), _vec(c)]}}

	var mesh := _mesh(node)

	return {"mesh": mesh} if mesh >= 0 else null

func _mesh(node: SceneGeometry) -> int:
	var at: int = node.mesh.index

	if _meshes.has(at):
		return _meshes[at]

	var surface := _scene.assets.mesh(at)

	if surface == null or surface.get_surface_count() == 0:
		_warn("%s: mesh %d is not in the scene, dropped" % [node.name, at])
		return -1

	var arrays := surface.surface_get_arrays(0)
	var vertices: PackedVector3Array = arrays[Mesh.ARRAY_VERTEX]

	var entry := {
		"vertices": _slice(vertices.to_byte_array()),
		"indices": _slice(_indices(arrays, vertices.size()).to_byte_array()),
	}

	for pair in [["normals", Mesh.ARRAY_NORMAL], ["tangents", Mesh.ARRAY_TANGENT],
			["uvs", Mesh.ARRAY_TEX_UV]]:
		var data = arrays[pair[1]]

		if data != null and not data.is_empty():
			entry[pair[0]] = _slice(data.to_byte_array())

	_json["meshes"].append(entry)
	_meshes[at] = _json["meshes"].size() - 1

	return _meshes[at]

func _indices(arrays: Array, vertex_count: int) -> PackedInt32Array:
	var indices := PackedInt32Array()

	if arrays[Mesh.ARRAY_INDEX] != null:
		indices = arrays[Mesh.ARRAY_INDEX]
	else:
		indices.resize(vertex_count)

		for i in vertex_count:
			indices[i] = i

	# Convert to CCW.
	var i := 0

	while i + 2 < indices.size():
		var keep := indices[i + 1]
		indices[i + 1] = indices[i + 2]
		indices[i + 2] = keep
		i += 3

	return indices

func _slice(bytes: PackedByteArray) -> Dictionary:
	while _bin.size() % 4 != 0:
		_bin.append(0)

	var at := _bin.size()
	_bin.append_array(bytes)

	return {"offset": at, "length": bytes.size()}

func _material(material: SceneMaterial) -> Dictionary:
	var entry := {
		"albedo": _channel(material.albedo, true),
		"metallic": _channel(material.metallic, false),
		"roughness": _channel(material.roughness, false),
		"emissive": _channel(material.emissive, true),
		"emissive_strength": material.emissive_strength,
		"transparency": material.transparency,
		"refractive_index": material.refractive_index,
	}

	var normal := _image(material.normal)

	if normal >= 0:
		entry["normal"] = {"map": normal, "scale": material.normal_scale}

	return entry

func _channel(channel: SceneChannel, rgb: bool) -> Dictionary:
	if channel.mapped():
		var map := _image(channel.map)

		if map >= 0:
			return {"map": map}

	if rgb:
		return {"constant": _linear(channel.rgb.constant)}

	return {"constant": channel.scalar.constant}

func _image(at: int) -> int:
	if at < 0:
		return -1

	if _images.has(at):
		return _images[at]

	var image := _scene.assets.image(at)

	if image == null:
		_warn("image %d is not in the scene, dropped" % at)
		return -1

	var entry := _slice(image.save_png_to_buffer())

	_json["images"].append(entry)
	_images[at] = _json["images"].size() - 1

	return _images[at]

func _parameters(scene: SceneRoot) -> Dictionary:
	const TEXTURE_FILTER := ["nearest", "bilinear", "mipmap_bilinear", "trilinear"]
	const REFLECTANCE := ["blinn_phong"]
	const SPLIT := ["median", "sah"]
	const TONEMAP := ["none", "reinhard"]

	return {
		"transform": {"method": "stepwise"},
		"rasterization": {
			"backface_cull": scene.backface_cull,
			"perspective_correct": scene.perspective_correct,
			"depth_test": scene.depth_test,
		},
		"shading": {
			"reflectance_model": REFLECTANCE[scene.reflectance_model],
			"texture_filter": TEXTURE_FILTER[scene.texture_filter],
		},
		"ray_tracing": {
			"max_depth": scene.max_depth,
			"shadows": scene.shadows,
			"reflections": scene.reflections,
			"refractions": scene.refractions,
			"acceleration": {
				"enabled": scene.bvh_enabled,
				"bvh_split": SPLIT[scene.bvh_split],
				"wireframe": scene.bvh_wireframe if scene.bvh_wireframe > 0 else null,
			},
		},
		"output": {
			"width": scene.width,
			"height": scene.height,
			"exposure": scene.exposure,
			"tonemap": TONEMAP[scene.tonemap],
		},
	}

func _vec(v: Vector3) -> Array:
	return [v.x, v.y, v.z]

func _linear(color: Color) -> Array:
	var c := color.srgb_to_linear()
	return [c.r, c.g, c.b]

func _warn(message: String) -> void:
	_warnings.append(message)
	push_warning(message)
