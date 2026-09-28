class_name ScaleGizmo extends TransformGizmo

const REACH := 0.7
const CUBE_SIZE := 0.1

const SNAP := 0.1

var _grabbed := Basis.IDENTITY
var _reach := 0.0

func _build() -> void:
	_to = REACH
	_shafts()

	var cube := BoxMesh.new()
	cube.size = Vector3.ONE * CUBE_SIZE

	for axis in 3:
		_handle(axis, cube, REACH)

func _on_begin(ray_origin: Vector3, ray_dir: Vector3) -> void:
	_grabbed = target.global_basis
	_reach = _along(ray_origin, ray_dir)

func _on_drag(ray_origin: Vector3, ray_dir: Vector3) -> void:
	var reach := _along(ray_origin, ray_dir)

	if is_nan(reach):
		return

	var factor := snappedf(reach / _reach, SNAP)

	# Zero would flatten the basis for good: scaled() multiplies, so there would be
	# nothing left to grow back.
	if is_zero_approx(factor):
		return

	var stretch := Vector3.ONE
	stretch[_axis] = factor

	target.global_basis = _grabbed.scaled(stretch)
