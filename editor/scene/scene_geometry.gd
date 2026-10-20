class_name SceneGeometry extends SceneObject

enum Kind {SPHERE, CUBE, MESH, TRIANGLE}

class Sphere:
	var geometry: SceneGeometry

	var radius: float = 0.5:
		set(value):
			radius = value
			geometry._rebuild_preview()

class Cube:
	var geometry: SceneGeometry

	var half_edge: float = 0.5:
		set(value):
			half_edge = value
			geometry._rebuild_preview()

class MeshFile:
	var geometry: SceneGeometry

	var source: String = "":
		set(value):
			source = value
			geometry._rebuild_preview()

	var mesh: int = 0:
		set(value):
			mesh = value
			geometry._rebuild_preview()

	var surface: int = 0:
		set(value):
			surface = value
			geometry._rebuild_preview()

class Triangle:
	var geometry: SceneGeometry

	var a: Vector3 = Vector3(-0.5, -0.5, 0.0):
		set(value):
			a = value
			geometry._rebuild_preview()

	var b: Vector3 = Vector3(0.5, -0.5, 0.0):
		set(value):
			b = value
			geometry._rebuild_preview()

	var c: Vector3 = Vector3(0.0, 0.5, 0.0):
		set(value):
			c = value
			geometry._rebuild_preview()

var kind: Kind = Kind.SPHERE:
	set(value):
		kind = value
		_rebuild_preview()

var sphere := Sphere.new()
var cube := Cube.new()
var mesh := MeshFile.new()
var triangle := Triangle.new()

@export var cast_shadows: bool = true

var material: SceneMaterial = SceneMaterial.new():
	set(value):
		if material.changed.is_connected(_apply_material):
			material.changed.disconnect(_apply_material)

		material = value if value != null else SceneMaterial.new()
		
		if not material.changed.is_connected(_apply_material):
			material.changed.connect(_apply_material)
		
		_apply_material()

var _preview: MeshInstance3D
var _preview_material := StandardMaterial3D.new()
var _mask: MeshInstance3D
# Kept here too: set_selected can land before _ready has built the silhouette,
# and the flag would be lost if it only ever lived on that instance.
var _is_selected := false

func _init() -> void:
	sphere.geometry = self
	cube.geometry = self
	mesh.geometry = self
	triangle.geometry = self

func _ready() -> void:
	super()

	_preview = MeshInstance3D.new()
	_preview.material_override = _preview_material
	add_child(_preview, false, Node.INTERNAL_MODE_FRONT)

	_mask = MeshInstance3D.new()
	_mask.layers = SELECTION_LAYER
	_mask.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	_mask.material_override = _mask_material()
	_mask.visible = _is_selected
	add_child(_mask, false, Node.INTERNAL_MODE_FRONT)

	if not material.changed.is_connected(_apply_material):
		material.changed.connect(_apply_material)
		
	_apply_material()

	_rebuild_preview()

func model():
	match kind:
		Kind.CUBE:
			return cube

		Kind.MESH:
			return mesh

		Kind.TRIANGLE:
			return triangle

	return sphere

func set_selected(value: bool) -> void:
	_is_selected = value

	if _mask != null:
		_mask.visible = value

func preview() -> MeshInstance3D:
	return _preview

func sync_preview(scene: SceneRoot) -> void:
	_preview_material.cull_mode = (BaseMaterial3D.CULL_DISABLED
		if scene != null and not scene.backface_cull else BaseMaterial3D.CULL_BACK)
	_preview_material.no_depth_test = scene != null and not scene.depth_test

	const TEXTURE_FILTER := {
		SceneRoot.TextureFilter.NEAREST: BaseMaterial3D.TEXTURE_FILTER_NEAREST,
		SceneRoot.TextureFilter.BILINEAR: BaseMaterial3D.TEXTURE_FILTER_LINEAR,
		SceneRoot.TextureFilter.MIPMAP_BILINEAR: BaseMaterial3D.TEXTURE_FILTER_LINEAR_WITH_MIPMAPS,
		SceneRoot.TextureFilter.TRILINEAR: BaseMaterial3D.TEXTURE_FILTER_LINEAR_WITH_MIPMAPS,
	}

	_preview_material.texture_filter = TEXTURE_FILTER[scene.texture_filter if scene != null
		else SceneRoot.TextureFilter.TRILINEAR]

func _rebuild_preview() -> void:
	if _preview == null:
		return

	_set_preview_mesh(_preview_mesh())

func _preview_mesh() -> Mesh:
	match kind:
		Kind.CUBE:
			return Tcc2Tessellate.cube(cube.half_edge)

		Kind.TRIANGLE:
			if (triangle.b - triangle.a).cross(triangle.c - triangle.a).length_squared() == 0.0:
				return null

			return Tcc2Tessellate.triangle(triangle.a, triangle.b, triangle.c)

		Kind.MESH:
			if mesh.source.is_empty():
				return null

			return Assets.surface(mesh.source, mesh.mesh, mesh.surface)

	return Tcc2Tessellate.sphere(sphere.radius)

func _set_preview_mesh(mesh: Mesh) -> void:
	_preview.mesh = mesh
	_mask.mesh = mesh

func _apply_material() -> void:
	if _preview_material != null:
		material.apply_to(_preview_material)

func _mask_material() -> StandardMaterial3D:
	var mask := StandardMaterial3D.new()
	mask.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	mask.albedo_color = Color.WHITE
	return mask
