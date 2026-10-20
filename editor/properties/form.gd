class_name Form extends VBoxContainer

var node: SceneObject
var binder := Binder.new()

func build(for_node: SceneObject) -> void:
	node = for_node

	binder.unbind()

	_build()

func _build() -> void:
	pass

func refresh_values() -> void:
	binder.refresh()

func bind(control: Node, object: Object, property: StringName) -> void:
	binder.bind(control, object, property)

func _exit_tree() -> void:
	binder.unbind()
