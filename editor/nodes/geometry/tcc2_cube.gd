class_name Tcc2Cube extends Tcc2Geometry

@export var half_edge: float = 0.5:
	set(value):
		half_edge = value
		_rebuild_preview()

func _rebuild_preview() -> void:
	if _preview == null:
		return

	var mesh := BoxMesh.new()
	mesh.size = Vector3.ONE * half_edge * 2.0
	_set_preview_mesh(mesh)
