package com.github.yoep.popcorn.backend.subtitles.model;

import lombok.Builder;

@Builder
public record SubtitleText(String text, boolean italic, boolean bold, boolean underline) {
}
