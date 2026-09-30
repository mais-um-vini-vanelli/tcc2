# Picks the property panel for the current selection.
extends VBoxContainer

static var PANELS: Array = [
	[Tcc2Scene, preload("res://properties/scene_form.tscn")],
	[Tcc2Light, preload("res://properties/light_form.tscn")],
	[Tcc2Camera, preload("res://properties/camera_form.tscn")],
	[Tcc2Geometry, preload("res://properties/geometry_form.tscn")],
	[Tcc2Node, preload("res://properties/group_form.tscn")],
]

var _panel: Tcc2Form

func show_node(node: Tcc2Node) -> void:
	if _panel != null:
		remove_child(_panel)
		_panel.queue_free()
		_panel = null

	if node == null:
		return

	for entry in PANELS:
		if is_instance_of(node, entry[0]):
			var scene := entry[1] as PackedScene

			if scene != null:
				_panel = scene.instantiate()
			else:
				_panel = (entry[1] as GDScript).new()

			break

	if _panel == null:
		return

	_panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	add_child(_panel)
	_panel.build(node)

func refresh_values() -> void:
	if _panel != null:
		_panel.refresh_values()
