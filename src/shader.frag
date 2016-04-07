#version 130
#extension GL_ARB_gpu_shader5 : require

precision highp float;
precision highp int;

in vec2 coord;
out vec4 out_color;
uniform sampler2D tex;

void main() {
    out_color = texture(tex, coord / 2 + vec2(0.5, 0.5));
}
