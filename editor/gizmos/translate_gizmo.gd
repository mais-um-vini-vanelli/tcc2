class_name TranslateGizmo extends TransformGizmo

const REACH := 1.5
const HEAD_LENGTH := 0.25
const HEAD_RADIUS := 0.08

const SNAP := 0.2

var _offset := 0.0

func _build() -> void:
	_from = REACH - HEAD_LENGTH
	_to = REACH

	var cone := CylinderMesh.new()
	cone.top_radius = 0.0
	cone.bottom_radius = HEAD_RADIUS
	cone.height = HEAD_LENGTH
	cone.radial_segments = 6
	cone.rings = 1

	for axis in 3:
		_handle(axis, cone, REACH - HEAD_LENGTH * 0.5)

func _on_begin(ray_origin: Vector3, ray_dir: Vector3) -> void:
	var reach := _along(ray_origin, ray_dir)
	_offset = 0.0 if is_nan(reach) else reach - _origin[_axis]

func _on_drag(ray_origin: Vector3, ray_dir: Vector3) -> void:
	var reach := _along(ray_origin, ray_dir)

	if is_nan(reach):
		return

	var place := _origin
	place[_axis] = snappedf(reach - _offset, SNAP)

	target.global_position = place
