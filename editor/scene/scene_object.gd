class_name SceneObject extends Node3D

const SELECTION_COLOR := Color("#ffd63d")
const ICONS_LAYER := 1 << 1
const SELECTION_LAYER := 1 << 2
const ICON_COLOR := Color("#e8eff7")
const ICON_SCREEN_HEIGHT := 0.04

func _ready() -> void:
	rotation_order = EULER_ORDER_ZYX

func set_selected(_value: bool) -> void:
	pass

func sync_preview(_scene: SceneRoot) -> void:
	pass

func _add_icon(texture: Texture2D) -> Sprite3D:
	var icon := Sprite3D.new()
	icon.texture = texture
	icon.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	icon.fixed_size = true
	icon.shaded = false
	
	# texture height = pixel_size * texture_height
	#
	# screen height = 2 * d * tan(fov/2)
	# screen height = 2 * 1 * tan(fov/2)  # fixed_size fixes d at 1
	# screen height = 2 * 1 * 1           # we assume fov = 90, so tan(fov/2) = 1
	#
	# fraction of screen height = texture height / screen height
	# fraction of screen height = pixel_size * texture_height / 2
	# pixel_size = fraction of screen height * 2 / texture_height
	icon.pixel_size = ICON_SCREEN_HEIGHT * 2 / texture.get_height()
	
	icon.modulate = ICON_COLOR
	icon.layers = ICONS_LAYER
	add_child(icon, false, Node.INTERNAL_MODE_FRONT)
	return icon

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

func find_scene() -> SceneRoot:
	var n: Node = self
	while n != null:
		if n is SceneRoot:
			return n
		n = n.get_parent()
	return null
