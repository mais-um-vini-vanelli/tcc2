class_name Import

var _path: String
var _state: GLTFState
var _cameras: bool
var _json := {}
var _warnings: Array[String] = []

static func glb(path: String, cameras: bool,
		warnings: Array[String] = []) -> SceneObject:
	var parsed := Assets.state(path)

	if parsed == null:
		return null

	var importer := Import.new()
	importer._path = path
	importer._state = parsed
	importer._cameras = cameras
	importer._json = parsed.json

	var group := SceneObject.new()
	group.name = path.get_file().get_basename().trim_suffix(".tcc2")

	for index in parsed.root_nodes:
		importer._build(index, group)

	warnings.append_array(importer._warnings)

	return group

func _build(node_index: int, parent: SceneObject) -> void:
	var gltf: GLTFNode = _state.get_nodes()[node_index]
	var node_name: String = gltf.resource_name

	if node_name.is_empty():
		node_name = "Node%d" % node_index

	var node := _node(gltf, node_index, node_name)
	node.name = node_name
	node.transform = gltf.xform
	parent.add_child(node)

	for child in gltf.children:
		_build(child, node)

func _node(gltf: GLTFNode, node_index: int, node_name: String) -> SceneObject:
	if gltf.mesh >= 0:
		return _mesh(gltf.mesh, node_name)

	if gltf.camera >= 0 and _cameras:
		_cameras = false
		return _camera(_state.get_cameras()[gltf.camera])

	if gltf.light >= 0:
		return _light(_state.get_lights()[gltf.light])
	else:
		var environment: Dictionary = _extras("nodes", node_index).get("environment", {})

		if not environment.is_empty():
			var light := SceneLight.new()
			light.kind = SceneLight.Kind.ENVIRONMENT

			var color = environment.get("color")

			if color != null and color is Array and color.size() >= 3:
				light.environment.color = Color(color[0], color[1], color[2]).linear_to_srgb()
			else:
				light.environment.color = Color.WHITE

			light.environment.intensity = float(environment.get("intensity", 1.0))
			return light

	return SceneObject.new()

func _mesh(mesh: int, node_name: String) -> SceneObject:
	var count: int = _state.get_meshes()[mesh].mesh.get_surface_count()

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

func _usable(mesh: int, index: int, node_name: String) -> bool:
	if Assets.surface(_path, mesh, index) != null:
		return true

	_warn("%s: mesh %d surface %d has no triangles, dropped"
		% [node_name, mesh, index])

	return false

func _warn_ignored(mesh: int, index: int, node_name: String) -> void:
	var arrays := Assets.surface(_path, mesh, index).surface_get_arrays(0)

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
	primitive.mesh.source = _path
	primitive.mesh.mesh = mesh
	primitive.mesh.surface = index
	primitive.material = _material(mesh, index)

	if index == 0:
		var geometry = _extras("meshes", mesh).get("geometry");
		
		if geometry is String and geometry == "triangle":
			var points := _triangle_points(mesh, index)

			if points.size() == 3:
				primitive.kind = SceneGeometry.Kind.TRIANGLE
				primitive.triangle.a = points[0]
				primitive.triangle.b = points[1]
				primitive.triangle.c = points[2]
		elif geometry != null and geometry is Dictionary:
			if geometry.has("sphere"):
				var sphere: Dictionary = geometry["sphere"]
				primitive.kind = SceneGeometry.Kind.SPHERE
				primitive.sphere.radius = float(sphere.get("radius", 1.0))
			elif geometry.has("cube"):
				var cube: Dictionary = geometry["cube"]
				primitive.kind = SceneGeometry.Kind.CUBE
				primitive.cube.half_edge = float(cube.get("half_edge", 0.5))

	return primitive

func _triangle_points(mesh: int, index: int) -> PackedVector3Array:
	var surface := Assets.surface(_path, mesh, index)

	if surface == null or surface.get_surface_count() == 0:
		return PackedVector3Array()

	var arrays := surface.surface_get_arrays(0)
	var vertices: PackedVector3Array = arrays[Mesh.ARRAY_VERTEX]
	var indices = arrays[Mesh.ARRAY_INDEX]

	if indices == null or indices.size() < 3:
		return PackedVector3Array()

	var points := PackedVector3Array()

	for i in [indices[0], indices[2], indices[1]]:
		if i < 0 or i >= vertices.size():
			return PackedVector3Array()

		points.append(vertices[i])

	return points

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
	model.intensity = gltf.intensity / Export.LUMENS_PER_WATT

	return node

func _material(mesh: int, index: int) -> SceneMaterial:
	var mat := _state.get_meshes()[mesh].mesh.get_surface_material(index)
	var id: int = _state.get_materials().find(mat)

	if id < 0 or not (mat is BaseMaterial3D):
		return SceneMaterial.new()

	var _mat := mat as BaseMaterial3D
	var material := SceneMaterial.new()

	var tint := Color(_mat.albedo_color, 1.0)
	var albedo := _texture(_mat.albedo_texture)

	if albedo == null:
		material.albedo.rgb.constant = tint
	else:
		material.albedo.kind = SceneChannel.Kind.TEXTURE
		material.albedo.texture = albedo
		material.albedo.rgb.factor = tint

	var constants: Array = _extras("materials", id).get("constants", [])

	_scalar(material.metallic, _mat.metallic, _texture(_mat.metallic_texture),
		_mat.metallic_texture_channel, "metallic" in constants)

	_scalar(material.roughness, _mat.roughness, _texture(_mat.roughness_texture),
		_mat.roughness_texture_channel, "roughness" in constants)

	if _mat.emission_enabled:
		var strength := _mat.emission_energy_multiplier
		var emissive := _texture(_mat.emission_texture)
		var emission := (_mat.emission.srgb_to_linear() * strength).linear_to_srgb()
		emission.a = 1.0

		if emissive == null:
			material.emissive.rgb.constant = emission
		else:
			material.emissive.kind = SceneChannel.Kind.TEXTURE
			material.emissive.texture = emissive
			material.emissive.rgb.factor = emission

	if _mat.normal_enabled:
		material.normal = _texture(_mat.normal_texture)
		material.normal_scale = _mat.normal_scale

	if _mat.uv1_offset != Vector3.ZERO or _mat.uv1_scale != Vector3.ONE:
		_warn("%s: a uv offset or scale was dropped" % _path)

	var extensions := _extensions("materials", id)

	material.transparency = float(extensions
		.get("KHR_materials_transmission", {}).get("transmissionFactor", 0.0))

	material.refractive_index = float(extensions
		.get("KHR_materials_ior", {}).get("ior", 1.5))

	return material

func _scalar(value: SceneChannel, factor: float,
		texture: TextureLocation, channel: int, padded: bool) -> void:
	if texture == null or padded:
		value.scalar.constant = factor
		return

	value.kind = SceneChannel.Kind.TEXTURE
	value.texture = texture
	value.scalar.channel = channel
	value.scalar.factor = factor

func _extensions(kind: String, index: int) -> Dictionary:
	var list: Array = _json.get(kind, [])

	if index < 0 or index >= list.size():
		return {}

	return list[index].get("extensions", {})

func _extras(kind: String, index: int) -> Dictionary:
	return _extensions(kind, index).get(Export.EXTENSION, {})

static func parameters(path: String, scene: SceneRoot) -> bool:
	var parsed := Assets.state(path)

	if parsed == null:
		return false

	var preset: Dictionary = parsed.json.get("extensions", {}) \
		.get(Export.EXTENSION, {}).get("parameters", {})

	if preset.is_empty():
		return false

	var pick := func(names: Array, value, fallback: int) -> int:
		var index: int = names.find(value)
		return index if index >= 0 else fallback

	var raster: Dictionary = preset.get("rasterization", {})
	scene.backface_cull = raster.get("backface_cull", scene.backface_cull)
	scene.perspective_correct = raster.get("perspective_correct",
		scene.perspective_correct)
	scene.depth_test = raster.get("depth_test", scene.depth_test)

	var shading: Dictionary = preset.get("shading", {})
	scene.reflectance_model = pick.call(["blinn_phong"],
		shading.get("reflectance_model"), scene.reflectance_model)
	scene.texture_filter = pick.call(
		["nearest", "bilinear", "mipmap_bilinear", "trilinear"],
		shading.get("texture_filter"), scene.texture_filter)

	var tracing: Dictionary = preset.get("ray_tracing", {})
	scene.max_depth = int(tracing.get("max_depth", scene.max_depth))
	scene.shadows = tracing.get("shadows", scene.shadows)
	scene.reflections = tracing.get("reflections", scene.reflections)
	scene.refractions = tracing.get("refractions", scene.refractions)

	var acceleration: Dictionary = tracing.get("acceleration", {})
	scene.bvh_enabled = acceleration.get("enabled", scene.bvh_enabled)
	scene.bvh_split = pick.call(["median", "sah"], acceleration.get("bvh_split"),
		scene.bvh_split)

	var wireframe: Variant = acceleration.get("wireframe")
	scene.bvh_wireframe = int(wireframe) if wireframe != null else scene.bvh_wireframe

	var output: Dictionary = preset.get("output", {})
	scene.width = int(output.get("width", scene.width))
	scene.height = int(output.get("height", scene.height))
	scene.exposure = float(output.get("exposure", scene.exposure))
	scene.tonemap = pick.call(["none", "reinhard"], output.get("tonemap"),
		scene.tonemap)

	return true

func _texture(tex: Texture2D) -> TextureLocation:
	if tex == null:
		return null

	var image: int = Assets.images(_path).find(tex)

	if image < 0:
		_warn("%s: a texture is not one of the file's images" % _path)
		return null

	var texture := TextureLocation.new()
	texture.source = _path
	texture.image = image

	return texture

func _warn(message: String) -> void:
	_warnings.append(message)
	push_warning(message)
