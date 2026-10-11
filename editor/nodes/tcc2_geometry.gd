class_name Tcc2Geometry extends Tcc2Node

enum Kind {SPHERE, CUBE, MESH}

class Sphere:
	var geometry: Tcc2Geometry

	var radius: float = 1.0:
		set(value):
			radius = value
			geometry._rebuild_preview()

class Cube:
	var geometry: Tcc2Geometry

	var half_edge: float = 0.5:
		set(value):
			half_edge = value
			geometry._rebuild_preview()

class MeshFile:
	var geometry: Tcc2Geometry

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

var kind: Kind = Kind.SPHERE:
	set(value):
		kind = value
		_rebuild_preview()

var sphere := Sphere.new()
var cube := Cube.new()
var mesh := MeshFile.new()

@export var cast_shadows: bool = true

var material: Tcc2Material = Tcc2Material.new():
	set(value):
		if material.changed.is_connected(_apply_material):
			material.changed.disconnect(_apply_material)

		material = value if value != null else Tcc2Material.new()
		
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

	return sphere

func set_selected(value: bool) -> void:
	_is_selected = value

	if _mask != null:
		_mask.visible = value

func preview() -> MeshInstance3D:
	return _preview

func _rebuild_preview() -> void:
	if _preview == null:
		return

	_set_preview_mesh(_preview_mesh())

func _preview_mesh() -> Mesh:
	match kind:
		Kind.CUBE:
			var box := BoxMesh.new()
			box.size = Vector3.ONE * cube.half_edge * 2.0
			return box

		Kind.MESH:
			if mesh.source.is_empty():
				return null

			return Tcc2Assets.surface(mesh.source, mesh.mesh, mesh.surface)

	var ball := SphereMesh.new()
	ball.radius = sphere.radius
	ball.height = sphere.radius * 2
	return ball

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
