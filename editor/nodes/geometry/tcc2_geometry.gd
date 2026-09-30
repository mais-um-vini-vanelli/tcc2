class_name Tcc2Geometry extends Tcc2Node

const OUTLINE_WIDTH := 0.04

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
var _selection: MeshInstance3D
# Kept here too: set_selected can land before _ready has built the outline, and
# the flag would be lost if it only ever lived on the node.
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

	_selection = MeshInstance3D.new()
	_selection.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	_selection.material_override = _selection_material()
	_selection.visible = _is_selected
	add_child(_selection, false, Node.INTERNAL_MODE_FRONT)

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

	if _selection != null:
		_selection.visible = value

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
	_selection.mesh = mesh

func _apply_material() -> void:
	if _preview_material != null:
		material.apply_to(_preview_material)

func _selection_material() -> StandardMaterial3D:
	# First pass: write to the stencil buffer.
	var mask := StandardMaterial3D.new()
	mask.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	mask.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	mask.albedo_color = Color(0, 0, 0, 0)
	mask.no_depth_test = true
	mask.stencil_mode = BaseMaterial3D.STENCIL_MODE_CUSTOM
	mask.stencil_flags = BaseMaterial3D.STENCIL_FLAG_WRITE
	mask.stencil_reference = 1
	mask.render_priority = 1

	# Second pass: write outline only to the fragments the first pass didn't touch.
	var outline := StandardMaterial3D.new()
	outline.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	outline.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	outline.albedo_color = SELECTION_COLOR
	outline.cull_mode = BaseMaterial3D.CULL_FRONT
	outline.grow = true
	outline.grow_amount = OUTLINE_WIDTH
	outline.no_depth_test = true
	outline.stencil_mode = BaseMaterial3D.STENCIL_MODE_CUSTOM
	outline.stencil_flags = BaseMaterial3D.STENCIL_FLAG_READ
	outline.stencil_compare = BaseMaterial3D.STENCIL_COMPARE_NOT_EQUAL
	outline.stencil_reference = 1
	outline.render_priority = 2

	mask.next_pass = outline
	return mask
