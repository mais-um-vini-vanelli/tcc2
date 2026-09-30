class_name ScaleGizmo extends TransformGizmo

const REACH := 0.7
const CUBE_SIZE := 0.1

const SNAP := 0.1

var _grabbed := Basis.IDENTITY
var _frame := Basis.IDENTITY
var _reach := 0.0

func _build() -> void:
	_to = REACH
	_shafts()

	var cube := BoxMesh.new()
	cube.size = Vector3.ONE * CUBE_SIZE

	for axis in 3:
		_handle(axis, cube, REACH)


func _orientation() -> Basis:
	return _frame if _axis >= 0 else target.global_basis.orthonormalized()

func _on_begin(ray_origin: Vector3, ray_dir: Vector3) -> void:
	_grabbed = target.basis
	_frame = target.global_basis.orthonormalized()
	_reach = _along(ray_origin, ray_dir)

func _on_drag(ray_origin: Vector3, ray_dir: Vector3) -> void:
	var reach := _along(ray_origin, ray_dir)

	if is_nan(reach):
		return

	var factor := snappedf(reach / _reach, SNAP)

	# Zero would flatten the basis for good: scaled_local() multiplies, so there
	# would be nothing left to grow back.
	if is_zero_approx(factor):
		return

	var stretch := Vector3.ONE
	stretch[_axis] = factor

	target.basis = _grabbed.scaled_local(stretch)
