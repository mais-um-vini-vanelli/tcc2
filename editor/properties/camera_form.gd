class_name CameraForm extends Form

const TITLES := ["Perspective", "Orthographic"]

func _build() -> void:
	var camera := node as SceneCamera
	var model: Object = camera.model()
	var orthographic := camera.kind == SceneCamera.Kind.ORTHOGRAPHIC

	%Camera.title = TITLES[camera.kind]

	%Camera.show_row(%fov, not orthographic)
	%Camera.show_row(%height, orthographic)

	if orthographic:
		bind(%height, model, &"height")
	else:
		bind(%fov, model, &"fov")

	bind(%near, model, &"near")
	bind(%far, model, &"far")

	%Transform.bind(binder, camera)
