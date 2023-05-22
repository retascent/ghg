// Re-maps the data from the texture using the metadata
vec4 channelValues(sampler2D dataMap, vec2 texturePoint, vec4 minValues, vec4 maxValues) {
    vec4 channels = texture(dataMap, texturePoint);
    vec4 ranges = maxValues - minValues;
    return (channels * ranges) + minValues;
}

float channelIndex(vec4 source, int channel) {
    if (channel == 0) {
        return source.r;
    } else if (channel == 1) {
        return source.g;
    } else if (channel == 2) {
        return source.b;
    } else if (channel == 3) {
        return source.a;
    }
    return -1.0;
}
