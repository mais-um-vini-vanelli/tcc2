# Turns a .glb into editor nodes: the file's tree comes in as a tree, with the
# glTF transforms kept as transforms and nothing baked or merged.
class_name Tcc2Import

var _path: String
var _state: GLTFState
var _cameras: bool
var _materials := {}

# Everything the file holds, under a single node named after it. Cameras come in
# only if the scene has none, since the lib renders through one.
static func glb(path: String, cameras: bool) -> Tcc2Node:
	var parsed := Tcc2Glb.state(path)

	if parsed == null:
		return null

	var importer := Tcc2Import.new()
	importer._path = path
	importer._state = parsed
	importer._cameras = cameras

	var group := Tcc2Node.new()
	group.name = path.get_file().get_basename()

	for index in parsed.root_nodes:
		importer._build(index, group)

	return group

func _build(index: int, parent: Tcc2Node) -> void:
	var gltf: GLTFNode = _state.get_nodes()[index]
	var node_name: String = gltf.resource_name

	if node_name.is_empty():
		node_name = "Node%d" % index

	var node := _node(gltf, node_name)
	node.name = node_name
	node.transform = gltf.xform
	parent.add_child(node)

	for child in gltf.children:
		_build(child, node)

func _node(gltf: GLTFNode, node_name: String) -> Tcc2Node:
	if gltf.mesh >= 0:
		return _mesh(gltf.mesh, node_name)

	if gltf.camera >= 0 and _cameras:
		_cameras = false
		return _camera(_state.get_cameras()[gltf.camera])

	if gltf.light >= 0:
		return _light(_state.get_lights()[gltf.light])

	# A group, or a camera there was no room for: the children still matter.
	return Tcc2Node.new()

# A primitive is one Object in the lib, so a mesh with several becomes a group.
func _mesh(mesh: int, node_name: String) -> Tcc2Node:
	var count: int = _state.get_meshes()[mesh].mesh.get_surface_count()

	if count == 1:
		return _primitive(mesh, 0)

	var group := Tcc2Node.new()

	for index in count:
		var primitive := _primitive(mesh, index)
		primitive.name = "%s_%d" % [node_name, index]
		group.add_child(primitive)

	return group

func _primitive(mesh: int, index: int) -> Tcc2Mesh:
	var primitive := Tcc2Mesh.new()
	primitive.source = _path
	primitive.mesh = mesh
	primitive.surface = index
	primitive.material = _material(mesh, index)
	return primitive

func _camera(gltf: GLTFCamera) -> Tcc2Camera:
	var node: Tcc2Camera

	if gltf.perspective:
		var perspective := Tcc2PerspectiveCamera.new()
		perspective.fov = rad_to_deg(gltf.fov)
		node = perspective
	else:
		var orthographic := Tcc2OrthographicCamera.new()
		orthographic.ortho_height = gltf.size_mag * 2.0
		node = orthographic

	node.near = gltf.depth_near
	node.far = gltf.depth_far

	return node

func _light(gltf: GLTFLight) -> Tcc2Light:
	var node: Tcc2Light

	match gltf.light_type:
		"directional":
			node = Tcc2DirectionalLight.new()

		"spot":
			var spot := Tcc2SpotLight.new()
			spot.inner_angle = rad_to_deg(gltf.inner_cone_angle)
			spot.outer_angle = rad_to_deg(gltf.outer_cone_angle)
			node = spot

		_:
			node = Tcc2PointLight.new()

	node.color = gltf.color
	node.intensity = gltf.intensity

	return node

# Primitives that share a glTF material share the Tcc2Material.
func _material(mesh: int, index: int) -> Tcc2Material:
	var mat := _state.get_meshes()[mesh].mesh.get_surface_material(index)
	var id: int = _state.get_materials().find(mat)

	if id < 0 or not (mat is BaseMaterial3D):
		return Tcc2Material.new()

	if _materials.has(id):
		return _materials[id]

	var _mat := mat as BaseMaterial3D
	var material := Tcc2Material.new()

	var tint := Color(_mat.albedo_color, 1.0)
	var albedo := _texture(_mat.albedo_texture)

	if albedo == null:
		material.albedo.constant = tint
	else:
		material.albedo.kind = Tcc2Value.Kind.TEXTURE
		material.albedo.texture = albedo

		if tint != Color.WHITE:
			push_warning("%s: albedo factor dropped beside its map" % _path)

	var metallic := _texture(_mat.metallic_texture)

	if metallic == null:
		material.metallic.constant = _mat.metallic
	else:
		material.metallic.kind = Tcc2Value.Kind.TEXTURE
		material.metallic.texture = metallic
		material.metallic.channel = _mat.metallic_texture_channel

		if not is_equal_approx(_mat.metallic, 1.0):
			push_warning("%s: metallic factor dropped beside its map" % _path)

	var roughness := _texture(_mat.roughness_texture)

	if roughness == null:
		material.roughness.constant = _mat.roughness
	else:
		material.roughness.kind = Tcc2Value.Kind.TEXTURE
		material.roughness.texture = roughness
		material.roughness.channel = _mat.roughness_texture_channel

		if not is_equal_approx(_mat.roughness, 1.0):
			push_warning("%s: roughness factor dropped beside its map" % _path)

	if _mat.emission_enabled:
		var strength := _mat.emission_energy_multiplier
		var emissive := _texture(_mat.emission_texture)

		if emissive == null:
			material.emissive.constant = _mat.emission * strength
		else:
			material.emissive.kind = Tcc2Value.Kind.TEXTURE
			material.emissive.texture = emissive

			if _mat.emission != Color.WHITE or not is_equal_approx(strength, 1.0):
				push_warning("%s: emissive factor dropped beside its map" % _path)

	if _mat.normal_enabled:
		material.normal = _texture(_mat.normal_texture)

		if not is_equal_approx(_mat.normal_scale, 1.0):
			push_warning("%s: normal scale %f dropped" % [_path, _mat.normal_scale])

	if _mat.uv1_offset != Vector3.ZERO or _mat.uv1_scale != Vector3.ONE:
		push_warning("%s: a uv offset or scale was dropped" % _path)

	_materials[id] = material
	return material

func _texture(tex: Texture2D) -> Tcc2Texture:
	if tex == null:
		return null

	var image: int = Tcc2Glb.images(_path).find(tex)

	if image < 0:
		push_warning("%s: a texture is not one of the file's images" % _path)
		return null

	var texture := Tcc2Texture.new()
	texture.source = _path
	texture.image = image

	return texture
