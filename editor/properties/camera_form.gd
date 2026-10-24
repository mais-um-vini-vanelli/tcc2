class_name CameraForm extends Form

const TITLES := ["Perspective", "Orthographic"]

func _build() -> void:
	var camera := node as SceneCamera
	var model: Object = camera.model()
	var orthographic := camera.kind == SceneCamera.Kind.ORTHOGRAPHIC

	%Camera.title = TITLES[camera.kind]

	%FovRow.visible = not orthographic
	%HeightRow.visible = orthographic

	if orthographic:
		bind(%Height, model, ^"height")
	else:
		bind(%Fov, model, ^"fov")

	bind(%Near, model, ^"near")
	bind(%Far, model, ^"far")

	%Transform.bind(self, camera)
