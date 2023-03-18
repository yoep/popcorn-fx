package com.github.yoep.popcorn.ui.view.conditions;

import org.springframework.context.annotation.Conditional;

import java.lang.annotation.*;

/**
 * {@link Conditional} that only matches when the TV mode is activated.
 */
@Target({ElementType.TYPE, ElementType.METHOD})
@Retention(RetentionPolicy.RUNTIME)
@Documented
@Conditional(OnTvModeCondition.class)
public @interface ConditionalOnTvMode {
}
