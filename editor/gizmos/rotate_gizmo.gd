class_name RotateGizmo extends TransformGizmo

const SEGMENTS := 24
const ARC := PI * 5.0 / 9.0
const HALF_WIDTH := 0.016

const SNAP := PI / 12.0

var _mesh := ImmediateMesh.new()
var _previous := 0.0
var _grabbed := Basis.IDENTITY

func _build() -> void:
	_add_mesh(_mesh)

# The rings are redrawn every frame, because which part of them shows depends on
# where the camera is.
func _process(delta: float) -> void:
	super(delta)

	if visible:
		_draw()

func _draw() -> void:
	var eye := camera.global_position - target.global_position

	_mesh.clear_surfaces()

	for axis in 3:
		var points := _arc(axis, LENGTH)

		_mesh.surface_begin(Mesh.PRIMITIVE_TRIANGLES, _materials[axis])

		for i in SEGMENTS:
			_band(points[i], points[i + 1], eye)

		_mesh.surface_end()

func _arc(axis: int, radius: float) -> Array[Vector3]:
	var normal: Vector3 = AXES[axis]
	var u: Vector3 = AXES[(axis + 1) % 3]
	var v: Vector3 = AXES[(axis + 2) % 3]

	var eye := (camera.global_position - target.global_position).normalized()
	var towards := eye - normal * eye.dot(normal)

	var span := ARC
	var middle := 0.0

	if towards.length() > 0.01:
		middle = atan2(towards.dot(v), towards.dot(u))
	else:
		span = TAU

	var points: Array[Vector3] = []

	for i in SEGMENTS + 1:
		var angle := middle - span * 0.5 + span * i / SEGMENTS
		points.append((u * cos(angle) + v * sin(angle)) * radius)

	return points

# One segment of a ring, widened into a ribbon turned to face the camera.
func _band(a: Vector3, b: Vector3, eye: Vector3) -> void:
	var side := (b - a).cross(eye).normalized() * HALF_WIDTH

	_mesh.surface_add_vertex(a - side)
	_mesh.surface_add_vertex(a + side)
	_mesh.surface_add_vertex(b + side)

	_mesh.surface_add_vertex(b + side)
	_mesh.surface_add_vertex(b - side)
	_mesh.surface_add_vertex(a - side)

func _outline(axis: int) -> Array[Vector3]:
	var points := _arc(axis, LENGTH * _screen_scale())
	var origin := target.global_position

	for i in points.size():
		points[i] += origin

	return points

func _on_begin(ray_origin: Vector3, ray_dir: Vector3) -> void:
	_grabbed = target.global_basis
	_previous = _angle(ray_origin, ray_dir)

func _on_drag(ray_origin: Vector3, ray_dir: Vector3) -> void:
	var current := _angle(ray_origin, ray_dir)

	if is_nan(current):
		return

	var turned = angle_difference(_previous, current)

	var axis: Vector3 = AXES[_axis]

	target.global_basis = Basis(axis, snappedf(turned, SNAP)) * _grabbed

func _angle(ray_origin: Vector3, ray_dir: Vector3) -> float:
	var axis: Vector3 = AXES[_axis]
	var hit = Plane(axis, _origin).intersects_ray(ray_origin, ray_dir)

	if hit == null:
		return NAN

	var offset: Vector3 = (hit as Vector3) - _origin
	var u: Vector3 = AXES[(_axis + 1) % 3]
	var v: Vector3 = AXES[(_axis + 2) % 3]

	return atan2(offset.dot(v), offset.dot(u))
