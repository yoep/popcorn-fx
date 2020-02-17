package com.github.yoep.video.vlc.conditions;

import org.springframework.context.annotation.Conditional;

import java.lang.annotation.*;

/**
 * {@link Conditional} that only matches when a VLC installation can be found back on the machine.
 */
@Target({ ElementType.TYPE, ElementType.METHOD })
@Retention(RetentionPolicy.RUNTIME)
@Documented
@Conditional(OnVlcCondition.class)
public @interface ConditionalOnVlcInstall {
}
