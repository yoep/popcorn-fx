package com.github.yoep.video.youtube.conditions;

import org.springframework.context.annotation.Conditional;

import java.lang.annotation.*;

/**
 * {@link Conditional} that only matches when the youtube video player has not been disabled by option {@link OnYoutubeVideoEnabled#DISABLE_OPTION}.
 */
@Target({ ElementType.TYPE, ElementType.METHOD })
@Retention(RetentionPolicy.RUNTIME)
@Documented
@Conditional(OnYoutubeVideoEnabled.class)
public @interface ConditionalOnYoutubeVideoEnabled {
}
