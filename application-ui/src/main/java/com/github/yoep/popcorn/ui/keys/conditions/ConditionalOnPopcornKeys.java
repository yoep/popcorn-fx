package com.github.yoep.popcorn.ui.keys.conditions;

import org.springframework.context.annotation.Conditional;

import java.lang.annotation.*;

/**
 * {@link Conditional} that only matches when the Popcorn Keys library is found.
 */
@Target({ ElementType.TYPE, ElementType.METHOD })
@Retention(RetentionPolicy.RUNTIME)
@Documented
@Conditional(OnPopcornKeys.class)
public @interface ConditionalOnPopcornKeys {
}
