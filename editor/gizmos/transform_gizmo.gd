class_name TransformGizmo extends Node3D

const AXES := [Vector3.RIGHT, Vector3.UP, Vector3.BACK]
const AXIS_COLORS := [Color("#ff3d6e"), Color("#2be882"), Color("#3a96ff")]

const LENGTH := 1.0
const SCREEN_SIZE := 0.26
const GRAB_PIXELS := 10.0
const HIGHLIGHT := 0.5

signal moved

var camera: Camera3D

var target: Tcc2Node = null:
	set(value):
		target = value
		_axis = -1
		highlight(-1)

var _axis := -1
var _hover := -1
var _origin := Vector3.ZERO
var _materials: Array[StandardMaterial3D] = []

# The stretch of axis this gizmo's handle occupies. The three take different
# stretches so that they never sit on top of one another, and the mesh and the hit
# test both measure from here.
var _from := 0.0
var _to := LENGTH

func _ready() -> void:
	visible = false

	var material = func(axis: int) -> StandardMaterial3D:
		var mat := StandardMaterial3D.new()
		mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		mat.cull_mode = BaseMaterial3D.CULL_DISABLED
		mat.albedo_color = AXIS_COLORS[axis]
		mat.no_depth_test = true
		mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
		mat.render_priority = 3
		return mat

	for axis in 3:
		_materials.append(material.call(axis))

	_build()

func _process(_delta: float) -> void:
	visible = _active()

	if not visible:
		return

	global_position = target.global_position
	global_basis = _orientation() * Basis.from_scale(Vector3.ONE * _screen_scale())

# Which handle the mouse is over, or -1.
func handle_at(screen_position: Vector2) -> int:
	if not _active() or camera.is_position_behind(target.global_position):
		return -1

	var found := -1
	var nearest := GRAB_PIXELS

	for axis in 3:
		var outline := _outline(axis)
		var previous := camera.unproject_position(outline[0])

		for i in range(1, outline.size()):
			var current := camera.unproject_position(outline[i])
			var closest := Geometry2D.get_closest_point_to_segment(screen_position, previous, current)
			var distance := screen_position.distance_to(closest)

			if distance < nearest:
				nearest = distance
				found = axis

			previous = current

	return found

# Everything a drag needs is taken once, here.
func begin(axis: int, ray_origin: Vector3, ray_dir: Vector3) -> void:
	_axis = axis
	_origin = target.global_position
	_on_begin(ray_origin, ray_dir)

	highlight(axis)

func _on_begin(_ray_origin: Vector3, _ray_dir: Vector3) -> void:
	pass

func end() -> void:
	_axis = -1

func drag(ray_origin: Vector3, ray_dir: Vector3) -> void:
	_on_drag(ray_origin, ray_dir)
	moved.emit()

func _on_drag(_ray_origin: Vector3, _ray_dir: Vector3) -> void:
	pass

func _build() -> void:
	pass

func _orientation() -> Basis:
	return Basis.IDENTITY

# The handle as a world-space polyline, which is all the hit test needs.
func _outline(axis: int) -> Array[Vector3]:
	var origin := target.global_position
	var direction: Vector3 = _orientation() * AXES[axis]
	var reach := _screen_scale()
	var points: Array[Vector3] = [origin + direction * _from * reach,
		origin + direction * _to * reach]
	return points

# How far along its axis the mouse ray falls, measured from the grab point.
func _along(ray_origin: Vector3, ray_dir: Vector3) -> float:
	# axis line:  _origin    + t · axis
	# mouse ray:  ray_origin + s · ray_dir
	#
	# gap between a point in each: g(t, s) = (_origin + t · axis) − (ray_origin + s · ray_dir)
	#									   = (_origin − ray_origin) + t · axis − s · ray_dir
	#                                      = d + t · axis − s · ray_dir
	#
	# we want 't' for the point where the mouse ray is the closest to the axis eg. where the gap is
	# the shortest. At the shortest gap, g is perpendicular to both lines; if it weren't perpendicular 
	# to the axis, you could slide along the axis and get closer.
	#
	# g · axis    = d·axis    + t·(axis·axis)    − s·(ray_dir·axis)    = 0
	# g · ray_dir = d·ray_dir + t·(axis·ray_dir) − s·(ray_dir·ray_dir) = 0
	#
	# with axis·axis = ray_dir·ray_dir = 1 and a = axis · ray_dir:
	#
	# g · axis    = d·axis    + t   − s·a = 0 -> t = -d·axis   + s·a
	# g · ray_dir = d·ray_dir + t·a − s   = 0 -> s = d·ray_dir + t·a
	#
	# replacing s in the equation for t:
	#
	# t           = -d·axis + (d·ray_dir + t·a)·a
	# t           = -d·axis + d·ray_dir·a + t·a·a
	# t − t·a·a   = -d·axis + d·ray_dir·a
	# t·(1 − a·a) = d·ray_dir·a − d·axis
	# t           = (d·ray_dir·a − d·axis) / (1 − a·a)
	var axis: Vector3 = _orientation() * AXES[_axis]
	var d := _origin - ray_origin
	var a := axis.dot(ray_dir)
	var denominator := 1.0 - a * a

	if absf(denominator) < 0.0001:
		# the mouse ray is parallel to the axis.
		return NAN

	return (ray_dir.dot(d) * a - axis.dot(d)) / denominator

func _active() -> bool:
	return camera != null and is_instance_valid(target)

func _screen_scale() -> float:
	var offset := target.global_position - camera.global_position

	# A perspective projection divides by the depth along the camera's forward axis,
	# not by the distance to the camera.
	var depth := maxf(-camera.global_basis.z.dot(offset), 0.001)
	return depth * tan(deg_to_rad(camera.fov) * 0.5) * SCREEN_SIZE

# Lights one of the three handles, or clears them all with -1.
func highlight(axis: int) -> void:
	if axis == _hover or _materials.is_empty():
		return

	_hover = axis

	for i in 3:
		var color: Color = AXIS_COLORS[i]
		_materials[i].albedo_color = color.lightened(HIGHLIGHT) if i == _hover else color

#region Helpers

func _shafts() -> void:
	var mesh := ImmediateMesh.new()

	for axis in 3:
		var direction: Vector3 = AXES[axis]

		mesh.surface_begin(Mesh.PRIMITIVE_LINES, _materials[axis])
		mesh.surface_add_vertex(direction * _from)
		mesh.surface_add_vertex(direction * _to)
		mesh.surface_end()

	_add_mesh(mesh)

func _handle(axis: int, mesh: Mesh, distance: float) -> void:
	var direction: Vector3 = AXES[axis]

	var instance := _add_mesh(mesh)
	instance.material_override = _materials[axis]
	instance.position = direction * distance

	if axis == 0:
		instance.rotation = Vector3(0, 0, -PI * 0.5)
	elif axis == 2:
		instance.rotation = Vector3(PI * 0.5, 0, 0)

func _add_mesh(mesh: Mesh) -> MeshInstance3D:
	var instance := MeshInstance3D.new()
	instance.mesh = mesh
	instance.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	add_child(instance)
	return instance

#endregion
