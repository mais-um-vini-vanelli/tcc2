class_name Tcc2PerspectiveCamera extends Tcc2Camera

@export_range(1.0, 179.0) var fov: float = 60.0:
	set(value):
		fov = value
		_rebuild_preview()

func _half_height() -> float:
	return tan(deg_to_rad(fov) * 0.5) * GIZMO_DEPTH
