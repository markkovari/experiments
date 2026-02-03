export class User {
  constructor(
    public readonly id: string,
    public readonly email: string,
    public readonly name: string,
    public readonly password: string,
    public readonly createdAt: Date,
    public readonly updatedAt: Date,
  ) {}

  static create(data: {
    id: string;
    email: string;
    name: string;
    password: string;
    createdAt?: Date;
    updatedAt?: Date;
  }): User {
    return new User(
      data.id,
      data.email,
      data.name,
      data.password,
      data.createdAt || new Date(),
      data.updatedAt || new Date(),
    );
  }

  // Business logic methods
  isPasswordValid(password: string): boolean {
    // In real app, use bcrypt.compare
    return this.password === password;
  }

  toDTO() {
    return {
      id: this.id,
      email: this.email,
      name: this.name,
      createdAt: this.createdAt,
      updatedAt: this.updatedAt,
    };
  }
}
