package com.github.yoep.popcorn.ui.view.conditions;

import org.springframework.context.annotation.Conditional;

import java.lang.annotation.*;

/**
 * {@link Conditional} that only matches when the desktop mode is activated.
 */
@Target({ ElementType.TYPE })
@Retention(RetentionPolicy.RUNTIME)
@Documented
@Conditional(OnDesktopModeCondition.class)
public @interface ConditionalOnDesktopMode {
}
