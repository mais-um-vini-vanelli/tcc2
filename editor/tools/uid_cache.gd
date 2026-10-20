extends SceneTree

# Reconstrói .godot/uid_cache.bin sem abrir o editor: varre os uids que já moram
# nos sidecars .uid, nos cabeçalhos .tscn/.tres e nos .import.

func _init() -> void:
	var map := {}
	_walk("res://", map)

	var by_id := {}

	for path in map:
		var id: int = ResourceUID.text_to_id(map[path])

		if id < 0:
			printerr("uid inválido em ", path, ": ", map[path])
			continue

		by_id[id] = path

	var file := FileAccess.open("res://.godot/uid_cache.bin", FileAccess.WRITE)

	if file == null:
		printerr("não deu para escrever o cache: ",
			error_string(FileAccess.get_open_error()))
		quit(1)
		return

	file.store_32(by_id.size())

	for id in by_id:
		var bytes := (by_id[id] as String).to_utf8_buffer()
		file.store_64(id)
		file.store_32(bytes.size())
		file.store_buffer(bytes)

	file.close()

	print("uids gravados: ", by_id.size())
	quit()

func _walk(at: String, map: Dictionary) -> void:
	var dir := DirAccess.open(at)

	if dir == null:
		return

	dir.list_dir_begin()
	var entry := dir.get_next()

	while entry != "":
		var full := at.path_join(entry)

		if dir.current_is_dir():
			if entry != ".godot":
				_walk(full, map)
		else:
			_record(full, map)

		entry = dir.get_next()

	dir.list_dir_end()

func _record(path: String, map: Dictionary) -> void:
	match path.get_extension().to_lower():
		"uid":
			var uid := FileAccess.get_file_as_string(path).strip_edges()

			if uid.begins_with("uid://"):
				map[path.trim_suffix(".uid")] = uid

		"import":
			var uid := _uid_in(path, "uid=")

			if uid != "":
				map[path.trim_suffix(".import")] = uid

		"tscn", "tres", "scn", "res":
			var uid := _uid_in(path, "[gd_")

			if uid != "":
				map[path] = uid

func _uid_in(path: String, lead: String) -> String:
	var file := FileAccess.open(path, FileAccess.READ)

	if file == null:
		return ""

	while not file.eof_reached():
		var line := file.get_line()

		if not line.begins_with(lead):
			continue

		var at := line.find("uid=\"")

		if at == -1:
			break

		var rest := line.substr(at + 5)

		return rest.substr(0, rest.find("\""))

	return ""
