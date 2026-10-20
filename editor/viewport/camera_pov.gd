class_name CameraPov extends Node

const ALL_LAYERS := 0xFFFFF

var scene_view: SubViewport
var container: SubViewportContainer
var editor_camera: Camera3D

var _camera: Camera3D
var _environment: Environment

var _saved := {}

func is_active() -> bool:
	return is_instance_valid(_camera)

## The scene camera the preview looks through, or null when there is none.
func render_camera() -> SceneCamera:
	var cameras := scene_view.find_children("*", "SceneCamera", true, false)
	return null if cameras.is_empty() else cameras[0]

func set_active(enabled: bool) -> void:
	var camera := render_camera() if enabled else null

	if is_instance_valid(_camera):
		_camera.get_parent().remove_child(_camera)
		_camera.queue_free()

	_camera = null
	_environment = null

	if camera == null:
		_restore()
		_pop()
		editor_camera.current = true
		return

	_save()

	_camera = Camera3D.new()
	# Icons, gizmos, the grid and the selection silhouettes sit on their own
	# layers, which the preview skips.
	_camera.cull_mask = (ALL_LAYERS
		& ~SceneObject.ICONS_LAYER & ~SceneObject.SELECTION_LAYER)
	_camera.environment = _build_environment()
	_environment = _camera.environment

	camera.add_child(_camera, false, Node.INTERNAL_MODE_FRONT)

	scene_view.msaa_3d = Viewport.MSAA_DISABLED
	container.stretch = false
	container.set_anchors_preset(Control.PRESET_TOP_LEFT)

	_sync()

	_camera.current = true

func _process(_delta: float) -> void:
	if is_active():
		_sync()

func _sync() -> void:
	var camera := _camera.get_parent() as SceneCamera
	var model = camera.model()
	var near := maxf(model.near, 0.001)

	if camera.kind == SceneCamera.Kind.ORTHOGRAPHIC:
		_camera.set_orthogonal(model.height, near, model.far)
	else:
		_camera.set_perspective(model.fov, near, model.far)

	var scene := _scene()

	if scene == null:
		return

	_environment.background_color = _environment_color()
	_environment.tonemap_exposure = pow(2.0, scene.exposure)
	_environment.tonemap_mode = (Environment.TONE_MAPPER_REINHARDT
		if scene.tonemap == SceneRoot.Tonemap.REINHARD
		else Environment.TONE_MAPPER_LINEAR)

	_push(scene)

	_fit(maxi(scene.width, 1), maxi(scene.height, 1))

## Renders at the output size and scales the result to whichever extent of the
## pane runs out first. Re-run every frame, so a window resize or an edit to the 
## output size lands immediately.
func _fit(width: int, height: int) -> void:
	var output := Vector2(width, height)
	var pane := container.get_parent_area_size()
	var factor := minf(pane.x / output.x, pane.y / output.y)

	scene_view.size = Vector2i(width, height)
	container.size = output
	container.scale = Vector2(factor, factor)
	container.position = ((pane - output * factor) * 0.5).floor()
	container.texture_filter = (CanvasItem.TEXTURE_FILTER_NEAREST if factor >= 1.0
		else CanvasItem.TEXTURE_FILTER_LINEAR)

func _push(scene: SceneRoot) -> void:
	for node in scene_view.find_children("*", "SceneObject", true, false):
		node.sync_preview(scene)

func _pop() -> void:
	for node in scene_view.find_children("*", "SceneObject", true, false):
		node.sync_preview(null)

func _environment_color() -> Color:
	var r := 0.0
	var g := 0.0
	var b := 0.0

	for light in scene_view.find_children("*", "SceneLight", true, false):
		if light.kind == SceneLight.Kind.ENVIRONMENT:
			var env = light.environment
			r += env.color.r * env.intensity
			g += env.color.g * env.intensity
			b += env.color.b * env.intensity

	return Color(r, g, b)

func _scene() -> SceneRoot:
	var found := scene_view.find_children("*", "SceneRoot", true, false)
	return null if found.is_empty() else found[0]

func _build_environment() -> Environment:
	var base: Environment = (scene_view.find_children("*", "WorldEnvironment", true, false)[0] as WorldEnvironment).environment

	var copy: Environment = base.duplicate()
	copy.background_mode = Environment.BG_COLOR
	copy.tonemap_white = 16.0

	return copy

func _save() -> void:
	if not _saved.is_empty():
		return

	_saved = {
		"msaa": scene_view.msaa_3d,
		"filter": container.texture_filter,
		"stretch": container.stretch,
		"scale": container.scale,
		"anchors": [container.anchor_left, container.anchor_top,
			container.anchor_right, container.anchor_bottom],
		"offsets": [container.offset_left, container.offset_top,
			container.offset_right, container.offset_bottom],
	}

func _restore() -> void:
	if _saved.is_empty():
		return

	container.scale = _saved["scale"]

	var anchors: Array = _saved["anchors"]
	container.anchor_left = anchors[0]
	container.anchor_top = anchors[1]
	container.anchor_right = anchors[2]
	container.anchor_bottom = anchors[3]

	var offsets: Array = _saved["offsets"]
	container.offset_left = offsets[0]
	container.offset_top = offsets[1]
	container.offset_right = offsets[2]
	container.offset_bottom = offsets[3]

	container.stretch = _saved["stretch"]
	container.texture_filter = _saved["filter"]
	scene_view.msaa_3d = _saved["msaa"]

	_saved = {}
