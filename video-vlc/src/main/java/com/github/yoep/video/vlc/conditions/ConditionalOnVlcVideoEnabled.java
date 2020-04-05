package com.github.yoep.video.vlc.conditions;

import org.springframework.context.annotation.Conditional;

import java.lang.annotation.*;

/**
 * {@link Conditional} that only matches when the VLC video player has not been disabled by option {@link OnVlcVideoEnabled#DISABLE_OPTION}.
 */
@Target({ElementType.TYPE, ElementType.METHOD})
@Retention(RetentionPolicy.RUNTIME)
@Documented
@Conditional(OnVlcVideoEnabled.class)
public @interface ConditionalOnVlcVideoEnabled {
}
