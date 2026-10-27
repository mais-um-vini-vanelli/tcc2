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
@onready var bvh: Button = $Layout/Viewport/Overlay/Panel/VBoxContainer/Bvh
@onready var trace_ray: Button = $Layout/Viewport/Overlay/Panel/VBoxContainer/TraceRay
@onready var help: Button = $Layout/Viewport/Overlay/Panel/VBoxContainer/Help

@onready var modes: Array[Button] = [camera_pov, pick_triangle, bvh, trace_ray]

const CONTROLS: Array[String] = [
	"LMB: select",
	"RMB hold + WASD/QE: fly",
	"Space + LMB drag: pan",
	"Esc: leave the current mode",
	"Drag and drop .glb file: import",
]

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
	sub_viewport_container.triangle_reported.connect(_on_triangle_reported)

	bvh.toggled.connect(_on_bvh_toggled)
	sub_viewport_container.bvh_reported.connect(_on_bvh_reported)

	trace_ray.toggled.connect(_on_trace_ray_toggled)
	sub_viewport_container.rays_reported.connect(_on_rays_reported)

	save_dialog = FileDialog.new()
	save_dialog.file_mode = FileDialog.FILE_MODE_SAVE_FILE
	save_dialog.access = FileDialog.ACCESS_FILESYSTEM
	save_dialog.title = "Save scene"
	save_dialog.use_native_dialog = true
	save_dialog.add_filter("*.tcc2", "tcc2 scene")
	save_dialog.file_selected.connect(_save_scene)
	add_child(save_dialog)

	open_dialog = FileDialog.new()
	open_dialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
	open_dialog.access = FileDialog.ACCESS_FILESYSTEM
	open_dialog.title = "Open scene"
	open_dialog.use_native_dialog = true
	open_dialog.add_filter("*.tcc2", "tcc2 scene")
	open_dialog.file_selected.connect(_open_scene)
	add_child(open_dialog)

	var on_save_pressed := func() -> void:
		save_dialog.current_file = root.name + ".tcc2"
		save_dialog.popup_centered_ratio(0.6)

	save.pressed.connect(on_save_pressed)
	open.pressed.connect(open_dialog.popup_centered_ratio.bind(0.6))

	help.pressed.connect(_show_controls)
	_show_controls()

func _show_controls() -> void:
	toast.report("CONTROLS", CONTROLS)

func _shortcut_input(event: InputEvent) -> void:
	if not event.is_action_pressed("ui_cancel"):
		return

	for button in modes:
		if button.button_pressed:
			button.button_pressed = false
			get_viewport().set_input_as_handled()

func _rebuild() -> void:
	hierarchy.rebuild(root)
	_on_scene_changed()

func _on_scene_changed() -> void:
	if sub_viewport_container.render_camera() == null:
		camera_pov.button_pressed = false
	elif camera_pov.button_pressed:
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

func _on_bvh_toggled(pressed: bool) -> void:
	_mode(bvh, pressed)

func _on_trace_ray_toggled(pressed: bool) -> void:
	_mode(trace_ray, pressed)

func _mode(entering: Button, pressed: bool) -> void:
	if pressed:
		for button in modes:
			if button != entering and button.button_pressed:
				button.button_pressed = false

		hierarchy.select(root)

	_apply_mode()
	_refresh_modes.call_deferred()

func _apply_mode() -> void:
	if pick_triangle.button_pressed:
		sub_viewport_container.set_mode(ViewportPane.Mode.PICK_TRIANGLE)
	elif camera_pov.button_pressed:
		sub_viewport_container.set_mode(ViewportPane.Mode.POV)
	elif bvh.button_pressed:
		sub_viewport_container.set_mode(ViewportPane.Mode.BVH)
	elif trace_ray.button_pressed:
		sub_viewport_container.set_mode(ViewportPane.Mode.TRACE_RAY)
	else:
		sub_viewport_container.set_mode(ViewportPane.Mode.EDIT)

func _refresh_modes() -> void:
	var busy := modes.any(func(button: Button) -> bool: return button.button_pressed)
	var blind := sub_viewport_container.render_camera() == null

	camera_pov.disabled = blind
	pick_triangle.disabled = blind
	trace_ray.disabled = blind

	hierarchy.set_editable(not busy)

func _on_triangle_reported(lines: Array[String]) -> void:
	toast.report("PICKED TRIANGLE", lines)

func _on_bvh_reported(lines: Array[String]) -> void:
	toast.report("BVH", lines)

func _on_rays_reported(lines: Array[String]) -> void:
	toast.report("TRACE RAY", lines)

func _save_scene(path: String) -> void:
	if path.get_extension().to_lower() != "tcc2":
		path += ".tcc2"

	var warnings: Array[String] = []
	var bytes := Write.to_tcc2(root, warnings)

	var file := FileAccess.open(path, FileAccess.WRITE)

	if file == null:
		push_error("could not write %s: %s"
			% [path, error_string(FileAccess.get_open_error())])
		return

	file.store_buffer(bytes)
	file.close()

	toast.report("SAVE WARNINGS", warnings)

	root.name = path.get_file().get_basename()
	hierarchy.rename(root)

func _open_scene(path: String) -> void:
	var warnings: Array[String] = []
	var opened := SceneRoot.new()
	opened.name = path.get_file().get_basename()

	if not Read.from_tcc2(path, opened, warnings):
		opened.free()
		return

	toast.report("OPEN WARNINGS", warnings)

	hierarchy.select(null)
	scene_view.remove_child(root)
	root.queue_free()

	root = opened
	scene_view.add_child(root)

	_rebuild()
	hierarchy.select(root)

func _on_files_dropped(files: PackedStringArray) -> void:
	var importing: Array[String] = ["processando..."]
	toast.report("IMPORT", importing)

	# The import blocks everything: let the toast land first.
	await get_tree().process_frame
	await get_tree().process_frame

	var warnings: Array[String] = []

	for file in files:
		if file.get_extension().to_lower() != "glb":
			continue

		var group := Import.glb(file, true, root.assets, warnings)

		if group == null:
			continue

		root.add_child(group)

		if Import.parameters(file, root):
			properties.refresh_values()

	toast.report("IMPORT WARNINGS", warnings)
	_rebuild()
