class_name Tcc2Export

const EXTENSION := "TCC2_extras"
const VERSION := 1

const FLOAT := 5126
const UNSIGNED_INT := 5125
const TRIANGLES := 4
const ARRAY_BUFFER := 34962
const ELEMENT_ARRAY_BUFFER := 34963

const SAMPLER := {
	"magFilter": 9729, # LINEAR
	"minFilter": 9987, # LINEAR_MIPMAP_LINEAR
	"wrapS": 10497, # REPEAT
	"wrapT": 10497, # REPEAT
}

var _json := {}
var _bin := PackedByteArray()
var _materials: Array[Tcc2Material] = []
var _material_ids: Array[int] = []
var _meshes := {}
var _images := {}
var _warnings: Array[String] = []

static func glb(scene: Tcc2Scene, warnings: Array[String] = []) -> PackedByteArray:
	var exporter := Tcc2Export.new()
	var bytes := exporter._build(scene)
	warnings.assign(exporter._warnings)
	return bytes

func _build(scene: Tcc2Scene) -> PackedByteArray:
	_json = {
		"asset": {"version": "2.0", "generator": "tcc2 editor"},
		"scene": 0,
		"scenes": [ {"nodes": []}],
		"nodes": [],
		"cameras": [],
		"meshes": [],
		"materials": [],
		"accessors": [],
		"bufferViews": [],
		"samplers": [],
		"textures": [],
		"images": [],
	}

	for child in scene.get_children():
		if child is Tcc2Node:
			var index := _node(child)

			if index >= 0:
				_json["scenes"][0]["nodes"].append(index)

	if not _json.has("extensions"):
		_json["extensions"] = {}

	_json["extensions"][EXTENSION] = {
		"version": VERSION,
		"parameters": _parameters(scene),
	}

	if _bin.size() > 0:
		_json["buffers"] = [ {"byteLength": _bin.size()}]

	var used := {
		"TCC2_extras": true,
		"KHR_lights_punctual": true,
		"KHR_materials_transmission": true,
		"KHR_materials_ior": true,
		"KHR_materials_emissive_strength": true
		}.keys()
	used.sort()

	_json["extensionsUsed"] = used

	for key in ["cameras", "meshes", "materials", "accessors", "bufferViews",
			"samplers", "textures", "images"]:
		if _json[key].is_empty():
			_json.erase(key)

	return _encode()

func _encode() -> PackedByteArray:
	var chunk := JSON.stringify(_json, "", false, true).to_utf8_buffer()

	while chunk.size() % 4 != 0:
		chunk.append(0x20)

	while _bin.size() % 4 != 0:
		_bin.append(0)

	var length := 12 + 8 + chunk.size()

	if _bin.size() > 0:
		length += 8 + _bin.size()

	var out := StreamPeerBuffer.new()
	out.big_endian = false

	out.put_u32(0x46546C67) # "glTF"
	out.put_u32(2)
	out.put_u32(length)

	out.put_u32(chunk.size())
	out.put_u32(0x4E4F534A) # "JSON"
	out.put_data(chunk)

	if _bin.size() > 0:
		out.put_u32(_bin.size())
		out.put_u32(0x004E4942) # "BIN\0"
		out.put_data(_bin)

	return out.data_array

func _node(node: Tcc2Node) -> int:
	var children: Array[int] = []

	for child in node.get_children():
		if child is Tcc2Node:
			var index := _node(child)

			if index >= 0:
				children.append(index)

	var entry := {}

	if node is Tcc2Camera:
		entry = _camera_node(node)
	elif node is Tcc2Light:
		entry = _light_node(node)
	elif node is Tcc2Geometry:
		entry = _geometry_node(node)

	if entry.is_empty():
		entry = _trs(node.transform)

	if not node.name.is_empty():
		entry["name"] = String(node.name)

	if not children.is_empty():
		entry["children"] = children

	_json["nodes"].append(entry)
	return _json["nodes"].size() - 1

func _trs(xform: Transform3D) -> Dictionary:
	var t := xform.origin
	var q := xform.basis.get_rotation_quaternion()
	var s := xform.basis.get_scale()

	return {
		"translation": [t.x, t.y, t.z],
		"rotation": [q.x, q.y, q.z, q.w],
		"scale": [s.x, s.y, s.z],
	}

func _matrix(xform: Transform3D) -> Array:
	var basis := xform.basis.orthonormalized()
	var x := basis * Vector3(1.0, 0.0, 0.0)
	var y := basis * Vector3(0.0, 1.0, 0.0)
	var z := basis * Vector3(0.0, 0.0, 1.0)
	var o := xform.origin

	return [
		x.x, x.y, x.z, 0.0,
		y.x, y.y, y.z, 0.0,
		z.x, z.y, z.z, 0.0,
		o.x, o.y, o.z, 1.0,
	]

func _camera_node(node: Tcc2Camera) -> Dictionary:
	var model = node.model()
	var camera := {}

	if node.kind == Tcc2Camera.Kind.PERSPECTIVE:
		camera = {
			"type": "perspective",
			"perspective": {
				"aspectRatio": node.aspect(),
				"yfov": deg_to_rad(model.fov),
				"znear": model.near,
				"zfar": model.far,
			},
		}
	else:
		var half_height: float = model.height * 0.5

		camera = {
			"type": "orthographic",
			"orthographic": {
				"xmag": half_height * node.aspect(),
				"ymag": half_height,
				"znear": model.near,
				"zfar": model.far,
			},
		}

	_json["cameras"].append(camera)

	return {
		"camera": _json["cameras"].size() - 1,
		"matrix": _matrix(node.transform),
	}

func _light_node(node: Tcc2Light) -> Dictionary:
	var model = node.model()
	var color: Color = model.color

	if node.kind == Tcc2Light.Kind.AMBIENT:
		var ambient := _trs(node.transform)

		ambient["extensions"] = {
			EXTENSION: {
				"ambient": {
					"color": _linear(color),
					"intensity": model.intensity,
				},
			},
		}

		return ambient

	var light := {
		"color": _linear(color),
		"intensity": model.intensity,
	}

	match node.kind:
		Tcc2Light.Kind.DIRECTIONAL:
			light["type"] = "directional"

		Tcc2Light.Kind.SPOT:
			light["type"] = "spot"
			light["spot"] = {
				"innerConeAngle": deg_to_rad(model.inner_angle),
				"outerConeAngle": deg_to_rad(model.outer_angle),
			}

		_:
			light["type"] = "point"

	if not _json.has("extensions"):
		_json["extensions"] = {}

	if not _json["extensions"].has("KHR_lights_punctual"):
		_json["extensions"]["KHR_lights_punctual"] = {"lights": []}

	var lights: Array = _json["extensions"]["KHR_lights_punctual"]["lights"]
	lights.append(light)

	return {
		"matrix": _matrix(node.transform),
		"extensions": {"KHR_lights_punctual": {"light": lights.size() - 1}},
	}

func _geometry_node(node: Tcc2Geometry) -> Dictionary:
	var mesh := _mesh(node)

	if mesh < 0:
		return {}

	var entry := _trs(node.transform)
	entry["mesh"] = mesh

	if not node.cast_shadows:
		entry["extensions"] = {EXTENSION: {"casts_shadows": false}}

	return entry

func _mesh(node: Tcc2Geometry) -> int:
	var material := _material(node.material)
	var key := ""

	if node.kind == Tcc2Geometry.Kind.MESH:
		key = "%s#%d#%d#%d" % [node.mesh.source, node.mesh.mesh,
			node.mesh.surface, material]

		if _meshes.has(key):
			return _meshes[key]

	var surface := _surface(node)

	if surface == null or surface.get_surface_count() == 0:
		return -1

	if surface is ArrayMesh \
			and surface.surface_get_primitive_type(0) != Mesh.PRIMITIVE_TRIANGLES:
		_warn("%s: not made of triangles, dropped" % node.name)
		return -1

	var arrays := surface.surface_get_arrays(0)
	var vertices: PackedVector3Array = arrays[Mesh.ARRAY_VERTEX]

	if vertices.is_empty():
		_warn("%s: no vertices, dropped" % node.name)
		return -1

	var attributes := {
		"POSITION": _accessor(vertices.to_byte_array(), vertices.size(), "VEC3",
			FLOAT, ARRAY_BUFFER, _bounds(vertices)),
	}

	var normals = arrays[Mesh.ARRAY_NORMAL]

	if normals != null and not normals.is_empty():
		attributes["NORMAL"] = _accessor(normals.to_byte_array(), normals.size(),
			"VEC3", FLOAT, ARRAY_BUFFER, null)

	var tangents = arrays[Mesh.ARRAY_TANGENT]

	if tangents != null and not tangents.is_empty():
		attributes["TANGENT"] = _accessor(tangents.to_byte_array(),
			tangents.size() / 4, "VEC4", FLOAT, ARRAY_BUFFER, null)

	var uvs = arrays[Mesh.ARRAY_TEX_UV]

	if uvs != null and not uvs.is_empty():
		attributes["TEXCOORD_0"] = _accessor(uvs.to_byte_array(), uvs.size(),
			"VEC2", FLOAT, ARRAY_BUFFER, null)

	if arrays[Mesh.ARRAY_TEX_UV2] != null:
		_warn("%s: a second uv set was dropped" % node.name)

	if arrays[Mesh.ARRAY_COLOR] != null:
		_warn("%s: vertex colors were dropped" % node.name)

	if arrays[Mesh.ARRAY_BONES] != null:
		_warn("%s: a skin was dropped" % node.name)

	var indices := _indices(arrays, vertices.size())

	var entry := {
		"primitives": [ {
			"attributes": attributes,
			"indices": _accessor(indices.to_byte_array(), indices.size(),
				"SCALAR", UNSIGNED_INT, ELEMENT_ARRAY_BUFFER, null),
			"material": material,
			"mode": TRIANGLES,
		}],
	}

	var tcc2_primitive = _tcc2_primitive(node)

	if tcc2_primitive != null:
		entry["extensions"] = {EXTENSION: {"primitive": tcc2_primitive}}

	_json["meshes"].append(entry)
	var index: int = _json["meshes"].size() - 1

	if not key.is_empty():
		_meshes[key] = index

	return index

func _surface(node: Tcc2Geometry) -> Mesh:
	match node.kind:
		Tcc2Geometry.Kind.CUBE:
			var box := BoxMesh.new()
			box.size = Vector3.ONE * node.cube.half_edge * 2.0
			return box

		Tcc2Geometry.Kind.MESH:
			if node.mesh.source.is_empty():
				_warn("%s: no mesh file, dropped" % node.name)
				return null

			var built := Tcc2Assets.surface(node.mesh.source, node.mesh.mesh,
				node.mesh.surface)

			if built == null:
				_warn("%s: %s has no mesh %d surface %d, dropped" % [node.name,
					node.mesh.source, node.mesh.mesh, node.mesh.surface])

			return built

	var ball := SphereMesh.new()
	ball.radius = node.sphere.radius
	ball.height = node.sphere.radius * 2.0
	return ball

func _tcc2_primitive(node: Tcc2Geometry):
	match node.kind:
		Tcc2Geometry.Kind.SPHERE:
			return {
				"sphere": {
					"center": [0.0, 0.0, 0.0],
					"radius": node.sphere.radius,
				},
			}

		Tcc2Geometry.Kind.CUBE:
			return {
				"cube": {
					"center": [0.0, 0.0, 0.0],
					"half_edge": node.cube.half_edge,
				},
			}

	return null

func _indices(arrays: Array, vertex_count: int) -> PackedInt32Array:
	var indices := PackedInt32Array()

	if arrays[Mesh.ARRAY_INDEX] != null:
		indices = arrays[Mesh.ARRAY_INDEX]
	else:
		indices.resize(vertex_count)

		for i in vertex_count:
			indices[i] = i

	# Godot's glTF importer swaps the last two indices of every triangle, to
	# turn glTF's counter-clockwise front faces into Godot's clockwise ones.
	var i := 0

	while i + 2 < indices.size():
		var keep := indices[i + 1]
		indices[i + 1] = indices[i + 2]
		indices[i + 2] = keep
		i += 3

	return indices

func _bounds(vertices: PackedVector3Array) -> Array:
	var low := vertices[0]
	var high := vertices[0]

	for v in vertices:
		low = low.min(v)
		high = high.max(v)

	return [[low.x, low.y, low.z], [high.x, high.y, high.z]]

func _accessor(bytes: PackedByteArray, count: int, type: String, component: int,
		target: int, bounds) -> int:
	var accessor := {
		"bufferView": _buffer_view(bytes, target),
		"byteOffset": 0,
		"componentType": component,
		"count": count,
		"type": type,
	}

	if bounds != null:
		accessor["min"] = bounds[0]
		accessor["max"] = bounds[1]

	_json["accessors"].append(accessor)
	return _json["accessors"].size() - 1

func _buffer_view(bytes: PackedByteArray, target: int) -> int:
	while _bin.size() % 4 != 0:
		_bin.append(0)

	var view := {
		"buffer": 0,
		"byteOffset": _bin.size(),
		"byteLength": bytes.size(),
	}

	if target > 0:
		view["target"] = target

	_bin.append_array(bytes)
	_json["bufferViews"].append(view)
	return _json["bufferViews"].size() - 1

func _material(material: Tcc2Material) -> int:
	var known := _materials.find(material)

	if known >= 0:
		return _material_ids[known]

	var pbr := {}

	if material.albedo.kind == Tcc2MaterialValue.Kind.TEXTURE:
		var texture := _texture(material.albedo.texture)

		if texture >= 0:
			pbr["baseColorFactor"] = [1.0, 1.0, 1.0, 1.0]
			pbr["baseColorTexture"] = {"index": texture}

	if not pbr.has("baseColorTexture"):
		var c := _linear(material.albedo.rgb.constant)
		pbr["baseColorFactor"] = [c[0], c[1], c[2], 1.0]

	var mr := _metallic_roughness(material)
	pbr["metallicFactor"] = mr[0]
	pbr["roughnessFactor"] = mr[1]

	if mr[2] >= 0:
		pbr["metallicRoughnessTexture"] = {"index": mr[2]}

	var entry := {
		"pbrMetallicRoughness": pbr,
		"doubleSided": true,
	}

	var strength := 1.0

	if material.emissive.kind == Tcc2MaterialValue.Kind.TEXTURE:
		var texture := _texture(material.emissive.texture)

		if texture >= 0:
			entry["emissiveFactor"] = [1.0, 1.0, 1.0]
			entry["emissiveTexture"] = {"index": texture}

	if not entry.has("emissiveTexture"):
		var e := material.emissive.rgb.constant.srgb_to_linear()
		
		strength = maxf(maxf(e.r, maxf(e.g, e.b)), 1.0)
		entry["emissiveFactor"] = [e.r / strength, e.g / strength, e.b / strength]

	var normal := _texture(material.normal)

	if normal >= 0:
		entry["normalTexture"] = {"index": normal, "scale": 1.0}

	var extensions := {}

	if material.transparency > 0.0:
		extensions["KHR_materials_transmission"] = {
			"transmissionFactor": material.transparency,
		}

	extensions["KHR_materials_ior"] = {"ior": material.refractive_index}

	if strength > 1.0:
		extensions["KHR_materials_emissive_strength"] = {
			"emissiveStrength": strength,
		}

	entry["extensions"] = extensions

	_json["materials"].append(entry)
	var index: int = _json["materials"].size() - 1

	_materials.append(material)
	_material_ids.append(index)

	return index

func _metallic_roughness(material: Tcc2Material) -> Array:
	var metallic := material.metallic
	var roughness := material.roughness

	var m_map := metallic.kind == Tcc2MaterialValue.Kind.TEXTURE
	var r_map := roughness.kind == Tcc2MaterialValue.Kind.TEXTURE

	var m_image := _source_image(metallic.texture) if m_map else null
	var r_image := _source_image(roughness.texture) if r_map else null

	if m_image == null and r_image == null:
		return [metallic.scalar.constant, roughness.scalar.constant, -1]

	if m_image != null and r_image != null \
			and metallic.texture.source == roughness.texture.source \
			and metallic.texture.image == roughness.texture.image \
			and metallic.scalar.channel == BaseMaterial3D.TEXTURE_CHANNEL_BLUE \
			and roughness.scalar.channel == BaseMaterial3D.TEXTURE_CHANNEL_GREEN:
		return [1.0, 1.0, _texture(metallic.texture)]

	var size := (r_image if r_image != null else m_image).get_size()

	for image in [m_image, r_image]:
		if image != null and image.get_size() != size:
			_warn("metallic and roughness share one texture in glTF: %dx%d resized to %dx%d"
				% [image.get_width(), image.get_height(), size.x, size.y])
			image.resize(int(size.x), int(size.y))

	var count := int(size.x) * int(size.y)
	var green := _channel(r_image, roughness.scalar.channel, count)
	var blue := _channel(m_image, metallic.scalar.channel, count)

	var packed := PackedByteArray()
	packed.resize(count * 3)

	for i in count:
		packed[i * 3 + 1] = green[i]
		packed[i * 3 + 2] = blue[i]

	var image := Image.create_from_data(int(size.x), int(size.y), false,
		Image.FORMAT_RGB8, packed)

	return [
		1.0 if m_map else metallic.scalar.constant,
		1.0 if r_map else roughness.scalar.constant,
		_texture_from_image(image),
	]

func _channel(image: Image, channel: int, count: int) -> PackedByteArray:
	var out := PackedByteArray()
	out.resize(count)

	if image == null:
		out.fill(255)
		return out

	var offset := 0

	match channel:
		BaseMaterial3D.TEXTURE_CHANNEL_GREEN:
			offset = 1

		BaseMaterial3D.TEXTURE_CHANNEL_BLUE:
			offset = 2

		BaseMaterial3D.TEXTURE_CHANNEL_ALPHA:
			offset = 3

		BaseMaterial3D.TEXTURE_CHANNEL_GRAYSCALE:
			offset = -1

	var data := image.get_data()

	if offset < 0:
		for i in count:
			out[i] = (int(data[i * 4]) + data[i * 4 + 1] + data[i * 4 + 2]) / 3
	else:
		for i in count:
			out[i] = data[i * 4 + offset]

	return out

func _texture(location: Tcc2TextureLocation) -> int:
	if location == null or location.source.is_empty():
		return -1

	var key := "%s#%d" % [location.source, location.image]

	if _images.has(key):
		return _images[key]

	var image := _source_image(location)

	if image == null:
		return -1

	var index := _texture_from_image(image)
	_images[key] = index

	return index

func _texture_from_image(image: Image) -> int:
	var rgb := Image.new()
	rgb.copy_from(image)

	if rgb.is_compressed():
		rgb.decompress()

	rgb.convert(Image.FORMAT_RGB8)

	var view := _buffer_view(rgb.save_png_to_buffer(), 0)

	_json["images"].append({"bufferView": view, "mimeType": "image/png"})

	if _json["samplers"].is_empty():
		_json["samplers"].append(SAMPLER.duplicate())

	_json["textures"].append({
		"sampler": 0,
		"source": _json["images"].size() - 1,
	})

	return _json["textures"].size() - 1

func _source_image(location: Tcc2TextureLocation) -> Image:
	if location == null or location.source.is_empty():
		return null

	var images: Array = Tcc2Assets.images(location.source)

	if location.image < 0 or location.image >= images.size():
		_warn("%s: image %d is not one of the file's" % [location.source,
			location.image])
		return null

	var texture: Texture2D = images[location.image]

	if texture == null:
		return null

	var image := texture.get_image()

	if image == null:
		return null

	var copy := Image.new()
	copy.copy_from(image)

	if copy.is_compressed():
		copy.decompress()

	copy.convert(Image.FORMAT_RGBA8)

	return copy

func _parameters(scene: Tcc2Scene) -> Dictionary:
	const FILTERING := ["nearest", "bilinear", "trilinear"]
	const FREQUENCY := ["flat", "gouraud", "phong"]
	const REFLECTANCE := ["blinn_phong"]
	const SPLIT := ["median", "sah"]
	const TONEMAP := ["none", "reinhard"]

	var background = null

	if scene.background_enabled:
		background = _linear(scene.background)

	return {
		"transform": {"method": "stepwise"},
		"rasterization": {
			"backface_cull": scene.backface_cull,
			"wireframe": scene.wireframe,
			"perspective_correct": scene.perspective_correct,
			"depth_test": scene.depth_test,
		},
		"clipping": {"enabled": scene.clipping_enabled},
		"texturing": {
			"mipmaps": scene.mipmaps,
			"filtering": FILTERING[scene.filtering],
		},
		"shading": {
			"frequency": FREQUENCY[scene.shading_frequency],
			"reflectance": REFLECTANCE[scene.reflectance_model],
		},
		"ray_tracing": {
			"max_depth": scene.max_depth,
			"shadows": scene.shadows,
			"reflections": scene.reflections,
			"refractions": scene.refractions,
			"acceleration": {
				"bvh": scene.bvh,
				"split": SPLIT[scene.bvh_split],
			},
		},
		"output": {
			"width": scene.width,
			"height": scene.height,
			"background": background,
			"exposure": scene.exposure,
			"tonemap": TONEMAP[scene.tonemap],
		},
	}

func _linear(color: Color) -> Array:
	var c := color.srgb_to_linear()
	return [c.r, c.g, c.b]

func _warn(message: String) -> void:
	_warnings.append(message)
	push_warning(message)
