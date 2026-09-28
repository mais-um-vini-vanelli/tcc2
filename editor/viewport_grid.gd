class_name ViewportGrid extends MeshInstance3D

const HALF := 10
const STEP := 1.0
const LIFT := 0.005

const GRID_COLOR := Color("#484848")
const X_COLOR := Color("#ea334c")
const Y_COLOR := Color("#70a726")
const Z_COLOR := Color("#2d83e8")

const AXIS_START := 0.1
const AXIS_LENGTH := 2.0

func _ready() -> void:
	cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF

	var m := ImmediateMesh.new()

	m.surface_begin(Mesh.PRIMITIVE_LINES, _material(GRID_COLOR))
	var extent := HALF * STEP
	for i in range(-HALF, HALF + 1):
		var o := i * STEP
		m.surface_add_vertex(Vector3(o, 0, -extent))
		m.surface_add_vertex(Vector3(o, 0, extent))
		m.surface_add_vertex(Vector3(-extent, 0, o))
		m.surface_add_vertex(Vector3(extent, 0, o))
	m.surface_end()

	_axis(m, Vector3.RIGHT, X_COLOR)
	_axis(m, Vector3.UP, Y_COLOR)
	_axis(m, Vector3.BACK, Z_COLOR)

	mesh = m

func _axis(m: ImmediateMesh, dir: Vector3, color: Color) -> void:
	var lift := Vector3.ZERO if dir == Vector3.UP else Vector3(0, LIFT, 0)

	m.surface_begin(Mesh.PRIMITIVE_LINES, _material(color))
	m.surface_add_vertex(dir * AXIS_START + lift)
	m.surface_add_vertex(dir * AXIS_LENGTH + lift)
	m.surface_end()

func _material(color: Color) -> StandardMaterial3D:
	var mat := StandardMaterial3D.new()
	mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	mat.albedo_color = color
	return mat
