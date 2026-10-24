class_name GizmoMesh

static func surface(mesh: ImmediateMesh, primitive: int, material: Material,
		vertices: PackedVector3Array) -> void:
	mesh.surface_begin(primitive, material)

	for vertex in vertices:
		mesh.surface_add_vertex(vertex)

	mesh.surface_end()

static func ring(radius: float, z: float, segments: int) -> PackedVector3Array:
	var points := PackedVector3Array()

	for i in segments:
		var a := TAU * i / segments
		points.append(Vector3(cos(a) * radius, sin(a) * radius, z))

	return points

## Points along a circle in the `u`/`v` plane, `from` one angle `to` another.
static func arc(u: Vector3, v: Vector3, radius: float, from: float, to: float,
		segments: int) -> PackedVector3Array:
	var points := PackedVector3Array()
	var step := (to - from) / segments

	for i in segments + 1:
		var angle := from + i * step
		points.append((u * cos(angle) + v * sin(angle)) * radius)

	return points

static func loop(points: PackedVector3Array) -> PackedVector3Array:
	var lines := PackedVector3Array()

	for i in points.size():
		lines.append_array([points[i], points[(i + 1) % points.size()]])

	return lines

## `points` widened into a ribbon of triangles turned to face `eye`.
static func ribbon(points: PackedVector3Array, eye: Vector3,
		half_width: float) -> PackedVector3Array:
	var triangles := PackedVector3Array()

	for i in points.size() - 1:
		var a := points[i]
		var b := points[i + 1]
		var side := (b - a).cross(eye).normalized() * half_width

		triangles.append_array([a - side, a + side, b + side,
			b + side, b - side, a - side])

	return triangles

static func frustum(near_half: Vector2, far_half: Vector2,
		depth: float) -> PackedVector3Array:
	var near := _corners(near_half, 0.0)
	var far := _corners(far_half, -depth)
	var lines := loop(far)

	if not near_half.is_zero_approx():
		lines.append_array(loop(near))

	for i in far.size():
		lines.append_array([near[i], far[i]])

	return lines

static func _corners(half: Vector2, z: float) -> PackedVector3Array:
	return PackedVector3Array([
		Vector3(-half.x, half.y, z),
		Vector3(half.x, half.y, z),
		Vector3(half.x, -half.y, z),
		Vector3(-half.x, -half.y, z),
	])
