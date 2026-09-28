class_name Tcc2Mesh extends Tcc2Geometry

@export_file("*.glb") var source: String = "":
	set(value):
		source = value
		_rebuild_preview()

# Which mesh of the file, and which of its primitives: one Tcc2Mesh is one of
# them, because one primitive is one Object in the lib.
@export var mesh: int = 0:
	set(value):
		mesh = value
		_rebuild_preview()

@export var surface: int = 0:
	set(value):
		surface = value
		_rebuild_preview()

func _rebuild_preview() -> void:
	if _preview == null:
		return

	var built: Mesh = null

	if not source.is_empty():
		built = Tcc2Glb.surface(source, mesh, surface)

	_set_preview_mesh(built)
