extends Control

@onready var hierarchy: Hierarchy = $Layout/Sidebar/Hierarchy
@onready var add_menu: AddMenu = $AddMenu
@onready var overlay: MarginContainer = $Layout/Viewport/Overlay
@onready var toast: Toast = $Layout/Viewport/Overlay/Toast
@onready var save: Button = $Layout/Viewport/Overlay/Panel/VBoxContainer/Save
@onready var open: Button = $Layout/Viewport/Overlay/Panel/VBoxContainer/Open
@onready var sub_viewport_container: ViewportPane = $Layout/Viewport/ViewportClip/SubViewportContainer
@onready var scene_view: SubViewport = $Layout/Viewport/ViewportClip/SubViewportContainer/SceneView
@onready var properties := $Layout/Sidebar/Properties/VBoxContainer/MarginContainer/ScrollContainer/Padding/VBoxContainer
@onready var camera_pov: Button = $Layout/Viewport/Overlay/Panel/VBoxContainer/CameraPOV
@onready var pick_triangle: Button = $Layout/Viewport/Overlay/Panel/VBoxContainer/PickTriangle
@onready var pick_pixel: Button = $Layout/Viewport/Overlay/Panel/VBoxContainer/PickPixel

const CURSORS: Array = [
	[preload("res://icons/Arrow.svg"), Input.CURSOR_ARROW, Vector2(6, 4)],
	[preload("res://icons/Hand.svg"), Input.CURSOR_POINTING_HAND, Vector2(8, 1)],
	[preload("res://icons/IBeam.svg"), Input.CURSOR_IBEAM, Vector2(12, 12)],
	[preload("res://icons/Forbidden.svg"), Input.CURSOR_FORBIDDEN, Vector2(12, 12)],
	[preload("res://icons/Pan.svg"), Input.CURSOR_MOVE, Vector2(11, 11)],
	[preload("res://icons/SplitH.svg"), Input.CURSOR_HSPLIT, Vector2(11, 5)],
	[preload("res://icons/SplitV.svg"), Input.CURSOR_VSPLIT, Vector2(5, 11)],
]

var save_dialog: FileDialog
var open_dialog: FileDialog

var root: SceneRoot

func _ready() -> void:
	get_window().theme = Palette.build()

	for cursor in CURSORS:
		Input.set_custom_mouse_cursor(Glyph.rimmed(cursor[0], Palette.INK),
			cursor[1], cursor[2])
	
	root = SceneRoot.new()
	root.name = "Scene"
	scene_view.add_child(root)

	var camera := SceneCamera.new()
	camera.kind = SceneCamera.Kind.PERSPECTIVE
	camera.name = "Perspective"
	camera.position = Vector3(0, 1, 4)
	root.add_child(camera)
	camera.rotation_degrees = Vector3(-14, 0, 0)

	var light := SceneLight.new()
	light.kind = SceneLight.Kind.DIRECTIONAL
	light.name = "Directional"
	light.position = Vector3(-2, 3, 2)
	root.add_child(light)
	light.rotation_degrees = Vector3(-45, -35, 0)

	var sphere := SceneGeometry.new()
	sphere.kind = SceneGeometry.Kind.SPHERE
	sphere.name = "Sphere"
	root.add_child(sphere)

	hierarchy.selected.connect(_on_selected)
	hierarchy.changed.connect(_on_scene_changed)
	hierarchy.add_pressed.connect(add_menu.open_below)
	add_menu.add_requested.connect(hierarchy.add_node)

	_rebuild()
	hierarchy.select(root)

	sub_viewport_container.scene_object_selected.connect(_on_scene_object_selected)
	get_window().files_dropped.connect(_on_files_dropped)
	sub_viewport_container.gizmo_moved.connect(properties.refresh_values)

	camera_pov.toggled.connect(_on_camera_pov_toggled)
	sub_viewport_container.overlay = overlay

	pick_triangle.toggled.connect(_on_pick_triangle_toggled)
	sub_viewport_container.triangle_picked.connect(_on_triangle_picked)

	pick_pixel.toggled.connect(_on_pick_pixel_toggled)
	sub_viewport_container.pixel_picked.connect(_on_pixel_picked)

	save_dialog = FileDialog.new()
	save_dialog.file_mode = FileDialog.FILE_MODE_SAVE_FILE
	save_dialog.access = FileDialog.ACCESS_FILESYSTEM
	save_dialog.title = "Save scene"
	save_dialog.use_native_dialog = true
	save_dialog.add_filter("*.tcc2.glb", "glTF binary")
	save_dialog.file_selected.connect(_save_scene)
	add_child(save_dialog)

	open_dialog = FileDialog.new()
	open_dialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
	open_dialog.access = FileDialog.ACCESS_FILESYSTEM
	open_dialog.title = "Open scene"
	open_dialog.use_native_dialog = true
	open_dialog.add_filter("*.tcc2.glb", "glTF binary")
	open_dialog.file_selected.connect(_open_scene)
	add_child(open_dialog)

	var on_save_pressed := func() -> void:
		save_dialog.current_file = root.name + ".tcc2.glb"
		save_dialog.popup_centered_ratio(0.6)

	save.pressed.connect(on_save_pressed)
	open.pressed.connect(open_dialog.popup_centered_ratio.bind(0.6))

func _rebuild() -> void:
	hierarchy.rebuild(root)
	_on_scene_changed()

func _on_scene_changed() -> void:
	if sub_viewport_container.render_camera() == null:
		camera_pov.button_pressed = false
	elif camera_pov.button_pressed or pick_pixel.button_pressed:
		sub_viewport_container.refresh_pov()

	_refresh_modes()

func _on_selected(node: SceneObject) -> void:
	sub_viewport_container.set_target(node)
	properties.show_node(node)

func _on_scene_object_selected(node: SceneObject) -> void:
	if node != null:
		hierarchy.select(node)

func _on_camera_pov_toggled(pressed: bool) -> void:
	_mode(camera_pov, pressed)

func _on_pick_triangle_toggled(pressed: bool) -> void:
	_mode(pick_triangle, pressed)

func _on_pick_pixel_toggled(pressed: bool) -> void:
	_mode(pick_pixel, pressed)

func _mode(entering: Button, pressed: bool) -> void:
	if pressed:
		for button in [camera_pov, pick_triangle, pick_pixel]:
			if button != entering and button.button_pressed:
				button.button_pressed = false

		hierarchy.select(root)

	_apply_mode()
	_refresh_modes.call_deferred()

func _apply_mode() -> void:
	if pick_pixel.button_pressed:
		sub_viewport_container.set_mode(ViewportPane.Mode.PICK_PIXEL)
	elif pick_triangle.button_pressed:
		sub_viewport_container.set_mode(ViewportPane.Mode.PICK_TRIANGLE)
	elif camera_pov.button_pressed:
		sub_viewport_container.set_mode(ViewportPane.Mode.POV)
	else:
		sub_viewport_container.set_mode(ViewportPane.Mode.EDIT)

func _refresh_modes() -> void:
	var busy := (camera_pov.button_pressed or pick_triangle.button_pressed
		or pick_pixel.button_pressed)
	var blind := sub_viewport_container.render_camera() == null

	camera_pov.disabled = blind
	pick_triangle.disabled = blind
	pick_pixel.disabled = blind

	hierarchy.set_editable(not busy)

func _on_triangle_picked(_node: SceneGeometry, instance: int, face: int) -> void:
	pick_triangle.button_pressed = false

	var picked: Array[String] = ["instance: %d" % instance, "face: %d" % face]

	toast.report("PICKED TRIANGLE", picked)

func _on_pixel_picked(x: int, y: int) -> void:
	pick_pixel.button_pressed = false

	var picked: Array[String] = ["pixel: (%d, %d)" % [x, y]]
	toast.report("PICKED PIXEL", picked)

func _save_scene(path: String) -> void:
	if path.get_extension().to_lower() != "glb":
		path += ".glb"

	var warnings: Array[String] = []
	var bytes := Export.glb(root, warnings)

	var file := FileAccess.open(path, FileAccess.WRITE)

	if file == null:
		push_error("could not write %s: %s"
			% [path, error_string(FileAccess.get_open_error())])
		return

	file.store_buffer(bytes)
	file.close()

	toast.report("SAVE WARNINGS", warnings)

	root.name = path.get_file().get_basename().trim_suffix(".tcc2")
	hierarchy.rename(root)

func _open_scene(path: String) -> void:
	var warnings: Array[String] = []
	var opened := Import.glb(path, true, warnings)

	if opened == null:
		return

	toast.report("OPEN WARNINGS", warnings)

	hierarchy.select(null)
	scene_view.remove_child(root)
	root.queue_free()

	root = SceneRoot.new()
	root.name = path.get_file().get_basename().trim_suffix(".tcc2")
	scene_view.add_child(root)

	for child in opened.get_children():
		opened.remove_child(child)
		root.add_child(child)

	opened.free()
	Import.parameters(path, root)

	_rebuild()
	hierarchy.select(root)

func _on_files_dropped(files: PackedStringArray) -> void:
	var warnings: Array[String] = []

	for file in files:
		if file.get_extension().to_lower() != "glb":
			continue

		var group := Import.glb(file, true, warnings)

		if group == null:
			continue

		root.add_child(group)

		if Import.parameters(file, root):
			properties.refresh_values()

	toast.report("IMPORT WARNINGS", warnings)
	_rebuild()
