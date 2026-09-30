shader_type spatial;
render_mode unshaded, cull_disabled, depth_draw_never, blend_mix;

uniform vec3 grid_color : source_color;
uniform vec3 x_color : source_color;
uniform vec3 z_color : source_color;
uniform float cell;
uniform float min_cell;
uniform float axis_alpha;
uniform float horizon;
uniform float width;

varying vec3 world;

void vertex() {
	world = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

float lines(vec2 p, float size, vec2 pixel) {
	vec2 q = p / size;
	vec2 step_uv = pixel / size;
	vec2 half_width = step_uv * width * 0.5;
	vec2 aa = step_uv * 0.75;

	vec2 d = abs(fract(q - 0.5) - 0.5);
	vec2 cover = 1.0 - smoothstep(max(half_width - aa, vec2(0.0)), half_width + aa, d);

	vec2 average = clamp(step_uv * width, vec2(0.0), vec2(1.0));
	cover = mix(cover, average, clamp(step_uv * 2.0 - 1.0, vec2(0.0), vec2(1.0)));

	return mix(cover.x, 1.0, cover.y);
}

float axis(float p, float pixel) {
	float half_width = pixel * width * 0.5;
	float aa = pixel * 0.75;
	return 1.0 - smoothstep(max(half_width - aa, 0.0), half_width + aa, abs(p));
}

void fragment() {
	vec2 p = world.xz;

	vec2 dx = dFdx(p);
	vec2 dy = dFdy(p);
	vec2 pixel = vec2(length(vec2(dx.x, dy.x)), length(vec2(dx.y, dy.y)));

	float level = max(log(max(pixel.x, pixel.y) * min_cell / cell) / log(10.0), 0.0);
	float size = cell * pow(10.0, floor(level));
	float blend = fract(level);

	float fine = lines(p, size, pixel) * (1.0 - blend);
	float coarse = lines(p, size * 10.0, pixel);

	vec3 color = grid_color;
	float alpha = max(fine, coarse);

	float along_x = axis(p.y, pixel.y);
	float along_z = axis(p.x, pixel.x);

	color = mix(color, x_color, along_x);
	alpha = mix(alpha, axis_alpha, along_x);
	color = mix(color, z_color, along_z);
	alpha = mix(alpha, axis_alpha, along_z);

	ALBEDO = color;
	ALPHA = alpha * smoothstep(0.0, horizon, abs(normalize(world - CAMERA_POSITION_WORLD).y));
}
