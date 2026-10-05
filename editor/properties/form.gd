class_name Tcc2Form extends VBoxContainer

const FIELD_THEME := preload("res://properties/field.tres")

var node: Tcc2Node
var binder := Tcc2Binder.new()

func build(for_node: Tcc2Node) -> void:
	node = for_node
	theme = FIELD_THEME

	binder.unbind()

	_build()

func _ready() -> void:
	var style := StyleBoxLine.new()
	style.color = Color("#30363d")
	style.grow_begin = -16.0
	style.grow_end = -16.0

	for child in get_children():
		if child is Tcc2Section and child.get_index() > 0:
			var line := HSeparator.new()
			line.add_theme_stylebox_override("separator", style)
			add_child(line)
			move_child(line, child.get_index())

func _build() -> void:
	pass

func refresh_values() -> void:
	binder.refresh()

func bind(control: Node, object: Object, property: StringName) -> void:
	binder.bind(control, object, property)

func _exit_tree() -> void:
	binder.unbind()
