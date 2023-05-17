#define M_PI 3.1415926535898

vec2 pointToUv(vec3 pointOnSphere) {
    float u = clamp(0.5 + atan(pointOnSphere.x, pointOnSphere.z) / 2.0 / M_PI, 0.0, 1.0);
    float v = clamp(0.5 + asin(pointOnSphere.y) / M_PI, 0.0, 1.0);
    return vec2(u, v);
}

vec3 uvToPoint(vec2 coordinate) {
    float y = sin(coordinate.x);
    float r = cos(coordinate.x);
    float x = sin(coordinate.y) * r;
    float z = -cos(coordinate.y) * r;
    return vec3(x, y, z);
}
