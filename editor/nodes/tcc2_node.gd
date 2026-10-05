class_name Tcc2Node extends Node3D

const SELECTION_COLOR := Color("#ffd63d")
const ICON_COLOR := Color("#e8eff7")
const ICON_SCREEN_HEIGHT := 0.05

func _ready() -> void:
	rotation_order = EULER_ORDER_XYZ

func set_selected(value: bool) -> void:
	pass

func _add_icon(texture: Texture2D) -> Sprite3D:
	var icon := Sprite3D.new()
	icon.texture = texture
	icon.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	icon.fixed_size = true
	icon.shaded = false
	# fraction of screen height = pixel_size × texture_height / 2
	icon.pixel_size = ICON_SCREEN_HEIGHT * 2.0 / texture.get_height()
	icon.modulate = ICON_COLOR
	add_child(icon, false, Node.INTERNAL_MODE_FRONT)
	return icon

func _line(mesh: ImmediateMesh, from: Vector3, to: Vector3) -> void:
	mesh.surface_add_vertex(from)
	mesh.surface_add_vertex(to)

func _tri(mesh: ImmediateMesh, a: Vector3, b: Vector3, c: Vector3) -> void:
	mesh.surface_add_vertex(a)
	mesh.surface_add_vertex(b)
	mesh.surface_add_vertex(c)

func _ring(radius: float, z: float, segments: int) -> Array[Vector3]:
	var points: Array[Vector3] = []
	for i in segments:
		var a := TAU * i / segments
		points.append(Vector3(cos(a) * radius, sin(a) * radius, z))
	return points

func _gizmo_material() -> StandardMaterial3D:
	var mat := StandardMaterial3D.new()
	mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	mat.cull_mode = BaseMaterial3D.CULL_DISABLED
	mat.albedo_color = Color.BLACK
	return mat

func _selected_gizmo_material() -> StandardMaterial3D:
	var mat := _gizmo_material()
	mat.albedo_color = SELECTION_COLOR
	mat.no_depth_test = true
	mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	mat.render_priority = 2
	return mat

func _set_icon_selected(icon: Sprite3D, value: bool, base_color: Color) -> void:
	icon.modulate = SELECTION_COLOR if value else base_color
	icon.no_depth_test = value
	icon.render_priority = 2 if value else 0

func find_scene() -> Tcc2Scene:
	var n: Node = self
	while n != null:
		if n is Tcc2Scene:
			return n
		n = n.get_parent()
	return null
