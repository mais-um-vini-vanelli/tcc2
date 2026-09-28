class_name Tcc2DirectionalLight extends Tcc2Light

func _light_preview() -> Light3D:
	return DirectionalLight3D.new()

func _gizmo_mesh() -> Mesh:
	const ARROW_LENGTH := 0.8
	const ARROW_HEAD_LENGTH := 0.3
	const ARROW_HEAD_RADIUS := 0.16
	const ARROW_SHAFT_RADIUS := 0.06
	const ARROW_SEGMENTS := 12
	const ARROW_RAYS := 4

	var tip := Vector3.FORWARD * ARROW_LENGTH
	var neck := - (ARROW_LENGTH - ARROW_HEAD_LENGTH)

	var head_rim := _ring(ARROW_HEAD_RADIUS, neck, ARROW_SEGMENTS)
	var shaft_far := _ring(ARROW_SHAFT_RADIUS, neck, ARROW_SEGMENTS)
	var shaft_near := _ring(ARROW_SHAFT_RADIUS, 0.0, ARROW_SEGMENTS)

	var mesh := ImmediateMesh.new()

	mesh.surface_begin(Mesh.PRIMITIVE_LINES, _gizmo_material())

	for i in head_rim.size():
		_line(mesh, head_rim[i], head_rim[(i + 1) % head_rim.size()])

	for i in shaft_near.size():
		_line(mesh, shaft_near[i], shaft_near[(i + 1) % shaft_near.size()])

	@warning_ignore("integer_division")
	var step := ARROW_SEGMENTS / ARROW_RAYS

	for i in ARROW_RAYS:
		var k := i * step
		_line(mesh, head_rim[k], tip)
		_line(mesh, head_rim[k], shaft_far[k])
		_line(mesh, shaft_far[k], shaft_near[k])

	mesh.surface_end()

	return mesh
