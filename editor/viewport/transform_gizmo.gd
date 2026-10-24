class_name TransformGizmo extends Node3D

signal moved

var camera: Camera3D:
	set(value):
		camera = value

		for mode in _modes:
			mode.camera = value

var target: SceneObject = null:
	set(value):
		target = value

		for mode in _modes:
			mode.target = value

var _modes: Array[Mode] = []
var _dragging: Mode = null

func _ready() -> void:
	for mode in [Translate.new(), Scale.new(), Rotate.new()]:
		add_child(mode)
		mode.camera = camera
		mode.target = target
		mode.moved.connect(moved.emit)
		_modes.append(mode)

func grab(screen_position: Vector2, ray_origin: Vector3, ray_dir: Vector3) -> bool:
	for mode in _modes:
		var axis := mode.handle_at(screen_position)

		if axis >= 0:
			_dragging = mode
			mode.begin(axis, ray_origin, ray_dir)
			return true

	return false

func drag(ray_origin: Vector3, ray_dir: Vector3) -> void:
	if _dragging != null:
		_dragging.drag(ray_origin, ray_dir)

func release() -> void:
	if _dragging != null:
		_dragging.end()
		_dragging = null

func is_dragging() -> bool:
	return _dragging != null

# Lights the handle under the mouse, at most one across the three modes.
func hover(screen_position: Vector2) -> void:
	var taken := false

	for mode in _modes:
		var axis := -1 if taken else mode.handle_at(screen_position)
		taken = taken or axis >= 0
		mode.highlight(axis)

class Mode extends Node3D:
	const AXES := [Vector3.RIGHT, Vector3.UP, Vector3.BACK]
	const AXES_COLORS := [Color("#ff3d6e"), Color("#2be882"), Color("#3a96ff")]

	const LENGTH := 1.0
	const SCREEN_SIZE := 0.26
	const GRAB_PIXELS := 10.0
	const HIGHLIGHT := 0.5

	signal moved

	var camera: Camera3D

	var target: SceneObject = null:
		set(value):
			target = value
			_axis = -1
			highlight(-1)

	var _axis := -1
	var _hover := -1
	var _origin := Vector3.ZERO
	var _materials: Array[StandardMaterial3D] = []

	# The stretch of axis this gizmo's handle occupies. The three take different
	# stretches so that they never sit on top of one another, and the mesh and the hit
	# test both measure from here.
	var _from := 0.0
	var _to := LENGTH

	func _ready() -> void:
		visible = false

		var material = func(axis: int) -> StandardMaterial3D:
			var mat := StandardMaterial3D.new()
			mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
			mat.cull_mode = BaseMaterial3D.CULL_DISABLED
			mat.albedo_color = AXES_COLORS[axis]
			mat.no_depth_test = true
			mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
			mat.render_priority = 3
			return mat

		for axis in 3:
			_materials.append(material.call(axis))

		_build()

	func _process(_delta: float) -> void:
		visible = _active()

		if not visible:
			return

		global_position = target.global_position
		global_basis = _orientation() * Basis.from_scale(Vector3.ONE * _screen_scale())

	# Which handle the mouse is over, or -1.
	func handle_at(screen_position: Vector2) -> int:
		if not _active() or camera.is_position_behind(target.global_position):
			return -1

		var found := -1
		var nearest := GRAB_PIXELS

		for axis in 3:
			var outline := _outline(axis)
			var previous := camera.unproject_position(outline[0])

			for i in range(1, outline.size()):
				var current := camera.unproject_position(outline[i])
				var closest := Geometry2D.get_closest_point_to_segment(screen_position, previous, current)
				var distance := screen_position.distance_to(closest)

				if distance < nearest:
					nearest = distance
					found = axis

				previous = current

		return found

	# Everything a drag needs is taken once, here.
	func begin(axis: int, ray_origin: Vector3, ray_dir: Vector3) -> void:
		_axis = axis
		_origin = target.global_position
		_on_begin(ray_origin, ray_dir)

		highlight(axis)

	func _on_begin(_ray_origin: Vector3, _ray_dir: Vector3) -> void:
		pass

	func end() -> void:
		_axis = -1

	func drag(ray_origin: Vector3, ray_dir: Vector3) -> void:
		_on_drag(ray_origin, ray_dir)
		moved.emit()

	func _on_drag(_ray_origin: Vector3, _ray_dir: Vector3) -> void:
		pass

	func _build() -> void:
		pass

	func _orientation() -> Basis:
		return Basis.IDENTITY

	# The handle as a world-space polyline, which is all the hit test needs.
	func _outline(axis: int) -> Array[Vector3]:
		var origin := target.global_position
		var direction: Vector3 = _orientation() * AXES[axis]
		var reach := _screen_scale()
		var points: Array[Vector3] = [origin + direction * _from * reach,
			origin + direction * _to * reach]
		return points

	# How far along its axis the mouse ray falls, measured from the grab point.
	func _along(ray_origin: Vector3, ray_dir: Vector3) -> float:
		# Teoria: apendice/reta-mais-proxima.md
		var axis: Vector3 = _orientation() * AXES[_axis]
		var d := _origin - ray_origin
		var a := axis.dot(ray_dir)
		var denominator := 1.0 - a * a

		if absf(denominator) < 0.0001:
			# the mouse ray is parallel to the axis.
			return NAN

		return (ray_dir.dot(d) * a - axis.dot(d)) / denominator

	func _active() -> bool:
		return camera != null and is_instance_valid(target)

	func _screen_scale() -> float:
		var offset := target.global_position - camera.global_position

		# A perspective projection divides by the depth along the camera's forward axis,
		# not by the distance to the camera.
		var depth := maxf(-camera.global_basis.z.dot(offset), 0.001)
		return depth * tan(deg_to_rad(camera.fov) * 0.5) * SCREEN_SIZE

	func highlight(axis: int) -> void:
		if axis == _hover or _materials.is_empty():
			return

		_hover = axis

		for i in 3:
			var color: Color = AXES_COLORS[i]
			_materials[i].albedo_color = color.lightened(HIGHLIGHT) if i == _hover else color

	#region Helpers

	func _shafts() -> void:
		var mesh := ImmediateMesh.new()

		for axis in 3:
			var direction: Vector3 = AXES[axis]

			mesh.surface_begin(Mesh.PRIMITIVE_LINES, _materials[axis])
			mesh.surface_add_vertex(direction * _from)
			mesh.surface_add_vertex(direction * _to)
			mesh.surface_end()

		_add_mesh(mesh)

	func _handle(axis: int, mesh: Mesh, distance: float) -> void:
		var direction: Vector3 = AXES[axis]

		var instance := _add_mesh(mesh)
		instance.material_override = _materials[axis]
		instance.position = direction * distance

		if axis == 0:
			instance.rotation = Vector3(0, 0, -PI * 0.5)
		elif axis == 2:
			instance.rotation = Vector3(PI * 0.5, 0, 0)

	func _add_mesh(mesh: Mesh) -> MeshInstance3D:
		var instance := MeshInstance3D.new()
		instance.mesh = mesh
		instance.layers = SceneObject.ICONS_LAYER
		instance.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		add_child(instance)
		return instance

	#endregion

class Translate extends Mode:
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

class Scale extends Mode:
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

class Rotate extends Mode:
	const SEGMENTS := 10
	const ARC := PI
	const HALF_WIDTH := 0.016

	const SNAP := PI / 12.0

	var _mesh := ImmediateMesh.new()
	var _previous := 0.0
	var _grabbed := Basis.IDENTITY

	func _build() -> void:
		_add_mesh(_mesh)

	# The rings are redrawn every frame, because which part of them shows depends on
	# where the camera is.
	func _process(delta: float) -> void:
		super(delta)

		if visible:
			_draw()

	func _draw() -> void:
		var eye := camera.global_position - target.global_position

		_mesh.clear_surfaces()

		for axis in 3:
			var points := _arc(axis, LENGTH)

			_mesh.surface_begin(Mesh.PRIMITIVE_TRIANGLES, _materials[axis])

			for i in SEGMENTS:
				_band(points[i], points[i + 1], eye)

			_mesh.surface_end()

	func _arc(axis: int, radius: float) -> Array[Vector3]:
		var normal: Vector3 = AXES[axis]
		var u: Vector3 = AXES[(axis + 1) % 3]
		var v: Vector3 = AXES[(axis + 2) % 3]

		var eye := (camera.global_position - target.global_position).normalized()
		var towards := eye - normal * eye.dot(normal)

		var span := ARC
		var middle := 0.0

		if towards.length() > 0.01:
			middle = atan2(towards.dot(v), towards.dot(u))
		else:
			span = TAU

		var points: Array[Vector3] = []
		
		var start := middle - span * 0.5
		var step := span / SEGMENTS

		for i in SEGMENTS + 1:
			var angle := start + i * step
			points.append((u * cos(angle) + v * sin(angle)) * radius)

		return points

	# One segment of a ring, widened into a ribbon turned to face the camera.
	func _band(a: Vector3, b: Vector3, eye: Vector3) -> void:
		var side := (b - a).cross(eye).normalized() * HALF_WIDTH

		_mesh.surface_add_vertex(a - side)
		_mesh.surface_add_vertex(a + side)
		_mesh.surface_add_vertex(b + side)

		_mesh.surface_add_vertex(b + side)
		_mesh.surface_add_vertex(b - side)
		_mesh.surface_add_vertex(a - side)

	func _outline(axis: int) -> Array[Vector3]:
		var points := _arc(axis, LENGTH * _screen_scale())
		var origin := target.global_position

		for i in points.size():
			points[i] += origin

		return points

	func _on_begin(ray_origin: Vector3, ray_dir: Vector3) -> void:
		_grabbed = target.global_basis
		_previous = _angle(ray_origin, ray_dir)

	func _on_drag(ray_origin: Vector3, ray_dir: Vector3) -> void:
		var current := _angle(ray_origin, ray_dir)

		if is_nan(current):
			return

		var turned = angle_difference(_previous, current)

		var axis: Vector3 = AXES[_axis]

		target.global_basis = Basis(axis, snappedf(turned, SNAP)) * _grabbed

	func _angle(ray_origin: Vector3, ray_dir: Vector3) -> float:
		var axis: Vector3 = AXES[_axis]
		var hit = Plane(axis, _origin).intersects_ray(ray_origin, ray_dir)

		if hit == null:
			return NAN

		var offset: Vector3 = (hit as Vector3) - _origin
		var u: Vector3 = AXES[(_axis + 1) % 3]
		var v: Vector3 = AXES[(_axis + 2) % 3]

		return atan2(offset.dot(v), offset.dot(u))
