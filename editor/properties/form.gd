class_name Tcc2Form extends VBoxContainer

var node: Tcc2Node
var binder := Tcc2Binder.new()

func build(for_node: Tcc2Node) -> void:
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
