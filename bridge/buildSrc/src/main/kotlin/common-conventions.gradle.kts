plugins {
    id("base-conventions")
    id("com.gradleup.shadow")
}

val shade: Configuration by configurations.creating
configurations.compileClasspath.get().extendsFrom(shade)
configurations.runtimeClasspath.get().extendsFrom(shade)

dependencies {
    shade(project(":common:all"))
}

tasks.shadowJar {
    configurations = listOf(shade)

    relocate("com.google.protobuf", "com.mkihr_ojisan.minecraftd_bridge.shaded.protobuf")
    relocate("kotlin", "com.mkihr_ojisan.minecraftd_bridge.shaded.kotlin")
    relocate("org.intellij", "com.mkihr_ojisan.minecraftd_bridge.shaded.intellij")
    relocate("org.jetbrains", "com.mkihr_ojisan.minecraftd_bridge.shaded.jetbrains")
}

tasks.named("assemble") {
    dependsOn("shadowJar")
}