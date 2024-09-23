#include<stdio.h>
#include<stdint.h>
#include<raylib.h>
#include<raymath.h>

#define WIDTH 800
#define WIDTHF32 ((float)WIDTH)
#define HEIGHT 600
#define HEIGHTF32 ((float)HEIGHT)
#define SHIP_ACC 65.0f
#define SHIP_STEER_SPEED 2.5f
#define SHIP_MIN_VELOCITY 0.75f
#define SHIP_FRICTION_ACC_MULTIPLIER (1.45f)
#define PI_HALF (PI * 0.5f)

typedef struct {
  Vector2 vertices[3];
  Vector2 pos;
  float rot; // rads
  float scale;
  Vector2 velocity;
  Vector2 transform[3];
} Ship;

typedef struct {
  Vector2* vertices;
  uint32_t vertex_count;
  Vector2* pos;
  float rot;
  float scale;
  Vector2* transform;
} Asteroid;

/// Puts the results in `Vector2* transform`
/// `rot` is in rads.
void transformVertices(float rot, float scale, Vector2 pos, Vector2* vertices, Vector2* transform, uint32_t vertex_count) {
  for (uint32_t i = 0; i < vertex_count; i++) {
    float s = sinf(rot);
    float c = cosf(rot);
    transform[i].x = (vertices[i].x * c - vertices[i].y * s) * scale + pos.x;
    transform[i].y = (vertices[i].y * c + vertices[i].x * s) * scale + pos.y;
  
  }
  
}

Vector2 vec2(float x, float y) {
  Vector2 ret = {
    .x = x,
    .y = y
  };
  return ret;
}

Vector2 wrapScreen(Vector2 v) {
  if (v.x < 0.0f) {
     v.x = WIDTHF32;
  }else if (v.x > WIDTHF32) {
    v.x = 0;
  }
  if (v.y < 0.0f) {
     v.y = HEIGHTF32;
  }else if (v.y > HEIGHTF32) {
    v.y = 0;
  }
  return v;
}

// RLAPI bool CheckCollisionPointTriangle(Vector2 point, Vector2 p1, Vector2 p2, Vector2 p3);
// RLAPI bool CheckCollisionPointPoly(Vector2 point, Vector2 *points, int pointCount);
int main(int argc, char** argv) {
  InitWindow(WIDTH, HEIGHT, "Asteroids");
  SetTargetFPS(60);

  Ship ship = {
    .vertices = {
      vec2(0., 1.),
      Vector2Normalize(vec2(-1., -1.)), 
      Vector2Normalize(vec2(1., -1.)),
    },
    .pos = vec2(WIDTHF32/2., HEIGHTF32/2.),
    .rot = PI * 0.5,
    .scale = 30.,
    .transform = {0}
  };

  while (!WindowShouldClose()) {
    BeginDrawing();
    ClearBackground(BLACK);
    float delta = GetFrameTime();

    if (IsKeyDown(KEY_RIGHT)) {
      ship.rot += SHIP_STEER_SPEED * delta;  
    }
    if (IsKeyDown(KEY_LEFT)) {
      ship.rot -= SHIP_STEER_SPEED * delta;
    }
    {
      Vector2 forward = Vector2Scale(vec2(cosf(ship.rot + PI_HALF), sinf(ship.rot + PI_HALF)), SHIP_ACC * delta);
      bool moving = false;
      if (IsKeyDown(KEY_UP)) {
        moving = true;
        ship.velocity = Vector2Add(ship.velocity, forward);
      }
      if (IsKeyDown(KEY_DOWN)) {
        moving = true;
        ship.velocity = Vector2Subtract(ship.velocity, forward);
      }

      if (!moving) {
        if (ship.velocity.x >= SHIP_MIN_VELOCITY) {
            ship.velocity.x -= SHIP_ACC * SHIP_FRICTION_ACC_MULTIPLIER * delta;
        } else if (ship.velocity.x <= -SHIP_MIN_VELOCITY) {
            ship.velocity.x += SHIP_ACC * SHIP_FRICTION_ACC_MULTIPLIER * delta;
        } else {
            ship.velocity.x = 0.;
        }

        if (ship.velocity.y >= SHIP_MIN_VELOCITY) {
            ship.velocity.y -= SHIP_ACC * SHIP_FRICTION_ACC_MULTIPLIER * delta;
        } else if (ship.velocity.y <= -SHIP_MIN_VELOCITY) {
            ship.velocity.y += SHIP_ACC * SHIP_FRICTION_ACC_MULTIPLIER * delta;
        } else {
            ship.velocity.y = 0.;
        }
      }
    }
    ship.pos = Vector2Add(ship.pos, Vector2Scale(ship.velocity, delta));
    ship.pos = wrapScreen(ship.pos);
    
    transformVertices(ship.rot, ship.scale, ship.pos, ship.vertices, ship.transform, 3);

    DrawTriangleLines(ship.transform[2], ship.transform[1], ship.transform[0], RED);

    EndDrawing();
  }  

  CloseWindow();
  return 0;
}
