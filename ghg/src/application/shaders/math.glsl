float max2(vec2 v) {
    return max(v.x, v.y);
}

float min2(vec2 v) {
    return min(v.x, v.y);
}

float max3(vec3 v) {
    return max(max2(v.xy), v.z);
}

float min3(vec3 v) {
    return min(min2(v.xy), v.z);
}

float max4(vec4 v) {
    return max(max3(v.wxy), v.z);
}

float min4(vec4 v) {
    return min(min3(v.wxy), v.z);
}
