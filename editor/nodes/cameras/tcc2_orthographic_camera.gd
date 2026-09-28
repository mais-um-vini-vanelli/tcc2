class_name Tcc2OrthographicCamera extends Tcc2Camera

@export var ortho_height: float = 2.0:
	set(value):
		ortho_height = value
		_rebuild_preview()

func _half_height() -> float:
	return ortho_height * 0.5

func _has_near_plane() -> bool:
	return true
