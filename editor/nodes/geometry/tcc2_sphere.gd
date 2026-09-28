class_name Tcc2Sphere extends Tcc2Geometry

@export var radius: float = 1.0:
	set(value):
		radius = value
		_rebuild_preview()

func _rebuild_preview() -> void:
	if _preview == null:
		return
	
	var mesh := SphereMesh.new()
	mesh.radius = radius
	mesh.height = radius * 2
	_set_preview_mesh(mesh)
