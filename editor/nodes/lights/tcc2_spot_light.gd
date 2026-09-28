class_name Tcc2SpotLight extends Tcc2Light

@export_range(0.0, 90.0) var inner_angle: float = 20.0
@export_range(0.0, 90.0) var outer_angle: float = 30.0:
	set(value):
		outer_angle = value
		_rebuild_preview()

func _light_preview() -> Light3D:
	var spot := SpotLight3D.new()
	spot.spot_angle = outer_angle
	return spot

func _gizmo_mesh() -> Mesh:
	const CONE_LENGTH := 0.8
	const CONE_SEGMENTS := 24
	const CONE_RAYS := 4

	var radius := CONE_LENGTH * tan(deg_to_rad(minf(outer_angle, 89.0)))
	var rim := _ring(radius, -CONE_LENGTH, CONE_SEGMENTS)

	var mesh := ImmediateMesh.new()

	mesh.surface_begin(Mesh.PRIMITIVE_LINES, _gizmo_material())

	for i in CONE_SEGMENTS:
		_line(mesh, rim[i], rim[(i + 1) % CONE_SEGMENTS])

	for i in CONE_RAYS:
		@warning_ignore("integer_division")
		_line(mesh, Vector3.ZERO, rim[i * CONE_SEGMENTS / CONE_RAYS])

	mesh.surface_end()

	return mesh
